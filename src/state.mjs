// Pure interpretation of observations. Failed reads must never become 'not configured'.
export function assess(snapshot) {
  if (!snapshot) return [];
  const knownPackages = Array.isArray(snapshot.packages);
  const installed = knownPackages && snapshot.packages.includes('com.gathin.tvhome');
  const active = typeof snapshot.home === 'string' && snapshot.home.startsWith('com.gathin.tvhome/');
  const wallpaper = snapshot.theme?.wallpaper;
  return [
    { name: 'Your home screen', state: active ? 'done' : snapshot.home == null ? 'unknown' : 'optional',
      detail: active ? 'Clean Your TV is already your home screen.' : snapshot.home == null ? 'The TV did not report its default home screen.' : installed ? 'Clean Your TV is installed. Another home screen is active.' : 'Your current home screen is in place. Replacement is optional.' },
    { name: 'Disabled apps', state: snapshot.disabled == null ? 'unknown' : 'done',
      detail: snapshot.disabled == null ? 'Couldn’t read disabled apps. No assumptions made.' : `${snapshot.disabled.length} apps are currently disabled. Their earlier history is unknown.` },
    { name: 'Wallpaper', state: typeof wallpaper !== 'boolean' ? 'unknown' : wallpaper ? 'done' : 'optional',
      detail: typeof wallpaper !== 'boolean' ? 'Couldn’t read wallpaper preferences.' : wallpaper ? `Wallpapers are enabled${snapshot.theme?.wallpaper_rotation_minutes ? ` · ${snapshot.theme.wallpaper_rotation_minutes} minute rotation` : ''}.` : 'Wallpapers are off. Keep the clean background or personalize later.' },
    { name: 'Online photo connection', state: 'unknown', detail: 'Private photo-provider settings aren’t visible here. Your existing configuration stays in place.' }
  ];
}
export function parseEndpoint(value) {
  const match = /^(\d{1,3}(?:\.\d{1,3}){3}):(\d{1,5})$/.exec(value.trim());
  if (!match) return false;
  const octets = match[1].split('.').map(Number), port = Number(match[2]);
  return octets.every(n => n <= 255) && port > 0 && port <= 65535 &&
    (octets[0] === 10 || (octets[0] === 192 && octets[1] === 168) ||
     (octets[0] === 172 && octets[1] >= 16 && octets[1] <= 31) ||
     (octets[0] === 169 && octets[1] === 254));
}
export const scenarios = {
  existing: { model: 'Living room TV', manufacturer: 'TCL', androidVersion: '12',
    home: 'com.gathin.tvhome/.MainActivity', packages: ['com.gathin.tvhome', 'org.example.player'],
    disabled: ['org.example.recommendations', 'org.example.analytics'],
    theme: {wallpaper: true, wallpaper_rotation_minutes: 30, wallpaper_dim: 40},
    isTv: true, errors: [], collectedAt: 'Demo snapshot' },
  fresh: { model: 'A new beginning', manufacturer: 'Android TV', androidVersion: '14',
    home: 'org.example.stock/.Home', packages: ['org.example.stock'], disabled: [],
    theme: null, isTv: true, errors: [], collectedAt: 'Demo snapshot' },
  interrupted: { model: 'Living room TV', manufacturer: 'Android TV', androidVersion: null,
    home: null, packages: null, disabled: null, theme: null, isTv: null,
    errors: ['Connection interrupted. Keep the TV awake and check its current debugging port.'], collectedAt: 'Demo snapshot' }
};
