// Cancellation invalidates results; native commands retain their own bounded lifetime.
export function createConnectionController(onChange, { timeoutMs = 65000 } = {}) {
  let generation = 0;
  let current = null;
  function publish(phase, label = '', error = '') {
    onChange({ phase, label, error, pending: !!current });
  }
  function cancel() {
    generation += 1;
    if (current) current.abort();
    current = null;
    publish('idle');
  }
  async function run(phase, label, work) {
    if (current) return false;
    const id = ++generation;
    const controller = new AbortController();
    current = controller;
    const valid = () => id === generation && !controller.signal.aborted;
    const context = {
      signal: controller.signal,
      active: valid,
      commit(update) { if (valid()) update(); },
      transition(next, text) { if (valid()) publish(next, text); }
    };
    publish(phase, label);
    let timer;
    let onAbort;
    try {
      const stopped = new Promise((_, reject) => {
        onAbort = () => reject(new Error('cancelled'));
        controller.signal.addEventListener('abort', onAbort, { once: true });
        timer = setTimeout(() => reject(new Error('The TV didn’t reply in time. Keep it awake and try again, or enter its current address.')), timeoutMs);
      });
      await Promise.race([Promise.resolve().then(() => work(context)), stopped]);
      if (!valid()) return false;
      current = null;
      publish('ready');
      return true;
    } catch (error) {
      if (!valid()) return false;
      generation += 1;
      current = null;
      controller.abort();
      publish('recoverable-error', '', String(error instanceof Error ? error.message : error));
      return false;
    } finally {
      clearTimeout(timer);
      controller.signal.removeEventListener('abort', onAbort);
    }
  }
  return { run, cancel };
}
