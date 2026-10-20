// Community reports are evidence to review, not authorization to disable apps.
const sourceCommit = 'e14d13ad096929ef033d3afd4e6c3fabca2953e8';
export const presetSources = {
  common: {path:'google-chromecast-gtv-4k', model:'Chromecast with Google TV 4K', firmware:'Android 14 · UTTC.250917.004'},
  tcl: {path:'tcl-qm6k-google-tv', model:'TCL QM6K Pro · G08', firmware:'Android 14 · AU16; firmware varies by region'},
  xiaomi: {path:'xiaomi-mi-tv-4a', model:'Xiaomi Mi TV 4A · MiTV-MOOQ1', firmware:'Android 10 · QTG3.200305.006.2591'},
  sony: {path:'sony-kd-55x85j', model:'Sony BRAVIA KD-55X85J · VH2', firmware:'Android 12 · South American firmware'},
  hisense: {path:'hisense-hismarttv-a4', model:'Hisense HiSmartTV A4', firmware:'Android TV 9 · PTMR.190127.037'},
  philips: {path:'philips-r8g-google-tv', model:'Philips R8G · whitefield', firmware:'Google TV 14 · UKRE.260524.058'}
};
export function presetEvidence(id) { return presetSources[id] || presetSources.common; }
export function sourceUrl(id) {
  return `https://github.com/Gathin23/cleanyourtv/blob/${sourceCommit}/examples/${presetSources[id].path}/debloat.txt`;
}
export const presets = [
 {id:'common',name:'Android TV / Google TV',note:'Shared Android packages found in community reports. Check the TV’s actual apps before deciding.'},
 {id:'tcl',name:'TCL Google TV',note:'Based on the QM6K Pro G08 report. Other TCL boards and regional firmware can behave differently.'},
 {id:'xiaomi',name:'Xiaomi / Redmi Android TV',note:'Based on the Mi TV 4A report. PatchWall and Google TV models have different home-screen dependencies.'},
 {id:'sony',name:'Sony BRAVIA Google TV',note:'Based on the KD-55X85J report. Packages and factory defaults vary by region.'},
 {id:'hisense',name:'Hisense Android TV',note:'Based on the HiSmartTV A4 report. VIDAA sets use a different system and are outside this app’s scope.'},
 {id:'philips',name:'Philips Google TV',note:'Based on the R8G report. Philips also sells TVs with other operating systems.'}
];
const entry = (packageId,title,group,impact,source,requiresCleanHome=false,features=[]) => ({packageId,title,group,impact,source,requiresCleanHome,features});
export const commonEntries = [
 entry('com.google.android.feedback','Google feedback','Diagnostics','Stops sending feedback reports from this app. This does not turn off every form of telemetry.','common'),
 entry('com.android.tv.feedbackconsent','TV feedback consent','Diagnostics','Removes this feedback-consent component; review its behavior on your firmware.','common'),
 entry('com.google.android.play.games','Google Play Games','Optional apps','Keep it if you play games or use its library and sign-in features.','common')
];
const brandEntries = {
 common: [],
 tcl: [
  entry('com.tcl.logkit','TCL LogKit','Diagnostics','Collects and uploads diagnostic logs. TCL support may rely on it.','tcl'),
  entry('com.tcl.ttvs','TCL viewing-data services','Viewing data','The reported package also contains T-Cast and Alexa components. Turning it off can remove those features.','tcl',false,['T-Cast','Alexa']),
  entry('com.tcl.tv.plus','TCL TV+','Optional apps','Removes access to TCL’s free streaming channels through this app.','tcl'),
  entry('com.tcl.exhibit','TCL retail demo','Store mode','Used for retail display features; confirm the TV is in home mode.','tcl')
 ],
 xiaomi: [
  entry('com.miui.tv.analytics','Xiaomi TV analytics','Viewing data','Disables this analytics package; other system services may still collect diagnostics.','xiaomi'),
  entry('com.xiaomi.statistic','Xiaomi statistics','Viewing data','Disables the statistics package listed in the Mi TV 4A report.','xiaomi'),
  entry('com.mitv.tvhome.atv','PatchWall content','Home recommendations','Can remove PatchWall content features. Check that a working replacement home screen is active first.','xiaomi',true),
  entry('com.xiaomi.mimusic2','Mi Music','Optional apps','Keep it if you listen through Mi Music.','xiaomi')
 ],
 sony: [
  entry('tv.samba.ssm','Samba TV viewing recognition','Viewing data','Recognizes what is on screen for viewing-data services. Package names differ across TVs.','sony'),
  entry('tv.samba.ssm.ui','Samba TV controls','Viewing data','The companion controls for Samba TV; review alongside the viewing-recognition service.','sony'),
  entry('com.sonyericsson.idd.agent','Sony usage-data agent','Diagnostics','Disables the usage-data agent listed in the BRAVIA report.','sony'),
  entry('com.sony.dtv.promos','Sony promotions','Home recommendations','Promotional content component; verify home-screen behavior on your firmware.','sony',true)
 ],
 hisense: [
  entry('com.hisense.app.logreport','Hisense log reports','Diagnostics','Collects diagnostic reports; support may request this service.','hisense'),
  entry('com.hisense.storemode','Hisense store mode','Store mode','Used for retail display behavior. Keep the TV in home mode.','hisense'),
  entry('com.seraphic.browser','Hisense browser','Optional apps','Keep it if you browse websites on the TV.','hisense')
 ],
 philips: [
  entry('tv.anoki.acr.anokiacroptin','Anoki viewing recognition','Viewing data','Recognizes content on screen for viewing-data services.','philips'),
  entry('com.apps.esticker3','Philips retail sticker','Store mode','Retail display overlay listed in the R8G report.','philips'),
  entry('com.realtek.leanbacklauncher.partnercustomizer','Partner home recommendations','Home recommendations','Changes partner content in the original home screen. Verify a working replacement home first.','philips',true)
 ]
};
export const keptFeatures = ['HDMI and live TV','Wi-Fi and Bluetooth','Remote and keyboard','Play Store and Play services','Casting and voice assistant','Screen reader','Firmware and remote updates'];
const protectedIds = new Set([
 'com.google.android.tungsten.setupwraith','com.google.android.apps.tv.launcherx','com.google.android.tvlauncher',
 'com.google.android.leanbacklauncher','com.google.android.gms','com.android.vending',
 'com.google.android.apps.mediashell','com.google.android.katniss','com.google.android.tv.remote.service',
 'com.google.android.tv.dfuservice','com.google.android.marvin.talkback','com.google.android.inputmethod.latin',
 'com.tcl.tv','com.tcl.tvinput','com.tcl.providers.config','com.tcl.initsetup','com.tcl.factory.view'
]);
export function suggestedPreset(snapshot) {
 const maker=(snapshot?.manufacturer || '').toLowerCase();
 return presets.find(p=>p.id!=='common' && maker.includes(p.id))?.id || 'common';
}
export function assessPresetEntry(item,snapshot) {
 const currentHome=snapshot?.home?.split('/')[0];
 if(protectedIds.has(item.packageId) || item.packageId===currentHome) return {...item,state:'kept',label:'Kept',reason:'Supports a protected feature or is the current home screen.'};
 if(!snapshot || !Array.isArray(snapshot.packages)) return {...item,state:'unknown',label:'Not checked',reason:'Connect a TV and finish the app check to compare this package.'};
 if(!snapshot.packages.includes(item.packageId)) return {...item,state:'absent',label:'Not installed',reason:'This package was not reported on this TV.'};
 if(!Array.isArray(snapshot.disabled)) return {...item,state:'unknown',label:'State unknown',reason:'The package is installed, but its enabled state could not be read.'};
 if(snapshot.disabled.includes(item.packageId)) return {...item,state:'done',label:'Already disabled',reason:'Already disabled on this TV. Its earlier history is unknown.'};
 if(snapshot.isTv !== true) return {...item,state:'kept',label:'Check compatibility',reason:'Android TV compatibility has not been confirmed.'};
 if(item.features?.length) return {...item,state:'conditional',label:'Review bundled features',reason:`Includes ${item.features.join(' and ')}. Keep it until you decide which of these features you use.`};
 if(item.requiresCleanHome && !snapshot.home?.startsWith('com.gathin.tvhome/')) return {...item,state:'conditional',label:'Check home screen first',reason:'A working replacement home screen must be verified before considering this package.'};
 return {...item,state:'review',label:'Available to review',reason:'Installed and enabled. Keep it if you use the feature.'};
}
export function presetRows(id,snapshot) {
 return [...commonEntries,...(brandEntries[id]||[])].map(item=>assessPresetEntry(item,snapshot));
}
