/** @OnlyCurrentDoc */
// Personal Drifter POC. Keep this spreadsheet private: editors can modify its script.
const CONFIG = Object.freeze({
  repo: 'yhaggai/kitor', baseBranch: 'v1',
  baseSha: '33928fb8a744ec0d5ebc29f7841e3f346098b194',
  sheetId: 950970064,
  csvPath: 'patches-workspace/drifter/he/payload/translation.csv',
  manifestPath: 'patches-workspace/drifter/he/patch.json'
});

function onOpen() {
  SpreadsheetApp.getUi().createMenu('Kitor')
    .addItem('Submit corrections…', 'showSubmission')
    .addItem('Disconnect GitHub', 'disconnectGitHub').addToUi();
}

function showSubmission() {
  SpreadsheetApp.getUi().showSidebar(HtmlService.createHtmlOutputFromFile('Sidebar')
    .setTitle('Submit Drifter corrections'));
}

function disconnectGitHub() {
  PropertiesService.getUserProperties().deleteProperty('githubToken');
  PropertiesService.getUserProperties().deleteProperty('device');
  SpreadsheetApp.getUi().alert('Local GitHub connection removed. You can also revoke the app in GitHub Settings → Applications.');
}

function authStatus() {
  return {connected: !!PropertiesService.getUserProperties().getProperty('githubToken'),
    configured: !!PropertiesService.getScriptProperties().getProperty('GITHUB_CLIENT_ID')};
}

function beginGitHubLogin() {
  const clientId = PropertiesService.getScriptProperties().getProperty('GITHUB_CLIENT_ID');
  if (!clientId) throw new Error('One-time setup required: configure GITHUB_CLIENT_ID for an OAuth app with Device Flow enabled.');
  const response = oauth_('https://github.com/login/device/code', {client_id: clientId, scope: 'public_repo'});
  if (!response.device_code) throw new Error(response.error_description || response.error || 'GitHub authorization could not start.');
  const device = {code: response.device_code, clientId: clientId,
    interval: response.interval || 5, expires: Date.now() + response.expires_in * 1000, next: 0};
  PropertiesService.getUserProperties().setProperty('device', JSON.stringify(device));
  return {code: response.user_code, url: 'https://github.com/login/device', interval: device.interval};
}

function finishGitHubLogin() {
  const props = PropertiesService.getUserProperties();
  const device = JSON.parse(props.getProperty('device') || 'null');
  if (!device || Date.now() >= device.expires) throw new Error('Authorization expired. Connect GitHub again.');
  if (Date.now() < device.next) return {pending: true, interval: device.interval};
  const result = oauth_('https://github.com/login/oauth/access_token', {
    client_id: device.clientId, device_code: device.code,
    grant_type: 'urn:ietf:params:oauth:grant-type:device_code'});
  if (result.access_token) {
    props.setProperty('githubToken', result.access_token); props.deleteProperty('device');
    return {connected: true};
  }
  if (result.error === 'slow_down') device.interval += 5;
  if (['slow_down', 'authorization_pending'].indexOf(result.error) >= 0) {
    device.next = Date.now() + device.interval * 1000;
    props.setProperty('device', JSON.stringify(device));
    return {pending: true, interval: device.interval};
  }
  props.deleteProperty('device');
  throw new Error(result.error_description || result.error || 'GitHub authorization failed.');
}

function oauth_(url, payload) {
  const r = UrlFetchApp.fetch(url, {method: 'post', payload: payload,
    headers: {Accept: 'application/json'}, muteHttpExceptions: true});
  return JSON.parse(r.getContentText());
}

function gh_(path, method, body, raw, allow404) {
  const token = PropertiesService.getUserProperties().getProperty('githubToken');
  if (!token) throw new Error('Connect GitHub first.');
  const options = {method: method || 'get', muteHttpExceptions: true,
    headers: {Authorization: 'Bearer ' + token,
      Accept: raw ? 'application/vnd.github.raw+json' : 'application/vnd.github+json',
      'X-GitHub-Api-Version': '2022-11-28'}};
  if (body !== undefined) {options.contentType = 'application/json'; options.payload = JSON.stringify(body);}
  const r = UrlFetchApp.fetch('https://api.github.com/' + path, options);
  if (allow404 && r.getResponseCode() === 404) return null;
  if (r.getResponseCode() < 200 || r.getResponseCode() >= 300) {
    if (r.getResponseCode() === 401) PropertiesService.getUserProperties().deleteProperty('githubToken');
    let message = 'GitHub request failed';
    try {message = JSON.parse(r.getContentText()).message || message;} catch (_) {}
    throw new Error(message + ' (HTTP ' + r.getResponseCode() + ')');
  }
  return raw ? r.getContentText() : JSON.parse(r.getContentText());
}

// Parse CSV while retaining each raw record, including its original line ending.
// Only rewritten rows change bytes; quoted multiline strings remain intact.
function parseCsv_(text) {
  const result = []; let start = 0, value = '', fields = [], quoted = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      if (quoted && text[i + 1] === '"') {value += '"'; i++;}
      else quoted = !quoted;
    } else if (c === ',' && !quoted) {fields.push(value); value = '';}
    else if ((c === '\r' || c === '\n') && !quoted) {
      fields.push(value); value = '';
      const ending = c === '\r' && text[i + 1] === '\n' ? '\r\n' : c;
      if (ending.length === 2) i++;
      result.push({fields: fields, raw: text.slice(start, i + 1), ending: ending});
      fields = []; start = i + 1;
    } else value += c;
  }
  if (quoted) throw new Error('Unterminated CSV quote.');
  if (start < text.length) {fields.push(value); result.push({fields: fields, raw: text.slice(start), ending: ''});}
  return result;
}

function encodeRow_(fields, ending) {
  return fields.map(v => /[",\r\n]/.test(v) ? '"' + v.replace(/"/g, '""') + '"' : v).join(',') + ending;
}

function applyRows_(original, input) {
  const records = parseCsv_(original);
  if (input.length !== records.length) throw new Error('Rows were added or removed. Restore the original row count before submitting.');
  const map = new Map();
  input.forEach((row, i) => {
    const normalized = row.map(x => x == null ? '' : String(x));
    if (normalized.length !== 6) throw new Error('Expected six columns at sheet row ' + (i + 1));
    const key = JSON.stringify(normalized.slice(0, 5));
    if (map.has(key)) throw new Error('Duplicate row identity at sheet row ' + (i + 1));
    map.set(key, normalized);
  });
  const changes = [];
  const csv = records.map((record, i) => {
    const before = record.fields, next = map.get(JSON.stringify(before.slice(0, 5)));
    if (!next) throw new Error('Source columns were edited or a row is missing (original record ' + (i + 1) + '). Only edit Hebrew.');
    if (next[5] === before[5]) return record.raw;
    if (i < 3) throw new Error('Headers, instructions, and translation settings cannot be submitted.');
    if (!next[5].trim()) throw new Error('Empty Hebrew correction for ' + before[4]);
    if (next[5].includes('\0')) throw new Error('A correction contains a NUL character.');
    const tokens = s => (s.match(/\{[^{}]+\}|<\/?[A-Za-z][^>]*>|%\d*\$?[sdif]/g) || []).sort().join('\n');
    if (tokens(before[5]) !== tokens(next[5])) throw new Error('Formatting tokens changed for ' + before[4]);
    changes.push({english: before[4], context: before[2] + ' / ' + before[3], before: before[5], after: next[5]});
    return encodeRow_(next, record.ending);
  }).join('');
  if (changes.length > 50) throw new Error('This POC accepts up to 50 changed lines per submission.');
  return {csv: csv, changes: changes};
}

function digest_(text) {
  return Utilities.computeDigest(Utilities.DigestAlgorithm.SHA_256, text, Utilities.Charset.UTF_8)
    .map(b => ('0' + ((b + 256) % 256).toString(16)).slice(-2)).join('');
}

function prepare_() {
  const sheet = SpreadsheetApp.getActive().getSheets().find(s => s.getSheetId() === CONFIG.sheetId);
  if (!sheet) throw new Error('Translation tab missing.');
  const range = sheet.getRange(1, 1, sheet.getLastRow(), 6);
  if (range.getFormulas().some(row => row.some(Boolean))) throw new Error('Formulas are not supported in the translation area. Use plain text.');
  const original = gh_('repos/' + CONFIG.repo + '/contents/' + CONFIG.csvPath + '?ref=' + CONFIG.baseSha, 'get', undefined, true);
  const result = applyRows_(original, range.getValues());
  result.digest = digest_(result.csv);
  return result;
}

function previewSubmission() {
  const result = prepare_();
  return {changes: result.changes, digest: result.digest};
}

function submitCorrections(expectedDigest) {
  const lock = LockService.getDocumentLock();
  if (!lock.tryLock(1000)) throw new Error('Another submission is running. Please retry shortly.');
  try {
    const data = prepare_();
    if (!data.changes.length) throw new Error('No changes to submit.');
    if (data.digest !== expectedDigest) throw new Error('The sheet changed after your preview. Review it again before submitting.');
    const user = gh_('user');
    const repo = gh_('repos/' + CONFIG.repo);
    if (!repo.permissions || !repo.permissions.push) throw new Error('This personal POC requires write access to yhaggai/kitor. Community fork submission is not enabled yet.');
    const branch = 'codex/drifter-sheet-' + user.login + '-' + digest_(CONFIG.baseSha + data.csv).slice(0, 16);
    const endpoint = 'repos/' + CONFIG.repo;
    const previous = gh_(endpoint + '/pulls?state=all&head=' + encodeURIComponent(CONFIG.repo.split('/')[0] + ':' + branch) + '&base=' + CONFIG.baseBranch);
    if (previous.length) return {url: previous[0].html_url, existing: true};
    const ref = gh_(endpoint + '/git/ref/heads/' + branch, 'get', undefined, false, true);
    if (!ref) {
      const manifest = JSON.parse(gh_(endpoint + '/contents/' + CONFIG.manifestPath + '?ref=' + CONFIG.baseSha, 'get', undefined, true));
      const file = manifest.files.find(f => f.source && f.source.path === 'payload/translation.csv');
      if (!file) throw new Error('Expected translation payload missing from manifest.');
      file.source.sha256 = data.digest;
      const base = gh_(endpoint + '/git/commits/' + CONFIG.baseSha);
      const files = [[CONFIG.csvPath, data.csv], [CONFIG.manifestPath, JSON.stringify(manifest, null, 2) + '\n']];
      const tree = files.map(([path, text]) => ({path: path, mode: '100644', type: 'blob',
        sha: gh_(endpoint + '/git/blobs', 'post', {content: Utilities.base64Encode(text, Utilities.Charset.UTF_8), encoding: 'base64'}).sha}));
      const createdTree = gh_(endpoint + '/git/trees', 'post', {base_tree: base.tree.sha, tree: tree});
      const commit = gh_(endpoint + '/git/commits', 'post', {message: 'POC: Drifter corrections submitted from Google Sheets', tree: createdTree.sha, parents: [CONFIG.baseSha]});
      gh_(endpoint + '/git/refs', 'post', {ref: 'refs/heads/' + branch, sha: commit.sha});
    }
    const pr = gh_(endpoint + '/pulls', 'post', {head: branch, base: CONFIG.baseBranch, draft: true,
      title: '[POC — do not merge] Drifter: ' + data.changes.length + ' correction(s) from Google Sheets',
      body: 'Submitted by @' + user.login + ' using the spreadsheet Submit button.\n\n'
        + data.changes.length + ' Hebrew entries changed. Source revision: `' + CONFIG.baseSha + '`. '
        + 'Other CSV records are preserved byte-for-byte; the payload checksum is updated.\n\n'
        + '**POC only: do not merge or publish.** Version and release revision are unchanged. '
        + 'No build or gameplay validation was run by this button. Drifter stores Hebrew in display order. '
        + 'The private spreadsheet URL is intentionally omitted.'});
    return {url: pr.html_url, existing: false};
  } finally {lock.releaseLock();}
}
