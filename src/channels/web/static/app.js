// IronClaw Web Gateway - Client

let token = '';
let eventSource = null;
let logEventSource = null;
let currentTab = 'chat';

console.log("%c Burunyuu~ 😼 IronClaw Neco-Arc Edition Loaded!", "color: #ffd700; font-size: 20px; background: #2b1d38; padding: 10px; border-radius: 10px;");

// --- Auth ---

function authenticate() {
  token = document.getElementById('token-input').value.trim();
  if (!token) {
    document.getElementById('auth-error').textContent = 'Token required';
    return;
  }

  // Test the token against the health-ish endpoint (chat/threads requires auth)
  apiFetch('/api/chat/threads')
    .then(() => {
      document.getElementById('auth-screen').style.display = 'none';
      document.getElementById('app').style.display = 'flex';
      connectSSE();
      connectLogSSE();
      loadHistory();
      loadThreads();
      loadMemoryTree();
      loadJobs();
      loadRoutines();
    })
    .catch((err) => {
      console.error('Auth failed:', err);
      document.getElementById('auth-error').textContent = 'Invalid token (Check console for details)';
    });
}

document.getElementById('token-input').addEventListener('keydown', (e) => {
  if (e.key === 'Enter') authenticate();
});

// --- API helper ---

function apiFetch(path, options) {
  const opts = options || {};
  opts.headers = opts.headers || {};
  opts.headers['Authorization'] = 'Bearer ' + token;
  if (opts.body && typeof opts.body === 'object') {
    opts.headers['Content-Type'] = 'application/json';
    opts.body = JSON.stringify(opts.body);
  }
  return fetch(path, opts).then((res) => {
    if (!res.ok) throw new Error(res.status + ' ' + res.statusText);
    return res.json();
  });
}

// --- SSE ---

function connectSSE() {
  if (eventSource) eventSource.close();

  eventSource = new EventSource('/api/chat/events?token=' + encodeURIComponent(token));

  eventSource.onopen = () => {
    document.getElementById('sse-dot').classList.remove('disconnected');
    document.getElementById('sse-status').textContent = 'Connected';
  };

  eventSource.onerror = () => {
    document.getElementById('sse-dot').classList.add('disconnected');
    document.getElementById('sse-status').textContent = 'Reconnecting...';
  };

  eventSource.addEventListener('response', (e) => {
    const data = JSON.parse(e.data);
    addMessage('assistant', data.content);
    setStatus('');
  });

  eventSource.addEventListener('thinking', (e) => {
    const data = JSON.parse(e.data);
    setStatus(data.message, true);
  });

  eventSource.addEventListener('tool_started', (e) => {
    const data = JSON.parse(e.data);
    setStatus('Running tool: ' + data.name, true);
  });

  eventSource.addEventListener('tool_completed', (e) => {
    const data = JSON.parse(e.data);
    const icon = data.success ? '\u2713' : '\u2717';
    setStatus('Tool ' + data.name + ' ' + icon);
  });

  eventSource.addEventListener('stream_chunk', (e) => {
    const data = JSON.parse(e.data);
    appendToLastAssistant(data.content);
  });

  eventSource.addEventListener('status', (e) => {
    const data = JSON.parse(e.data);
    setStatus(data.message);
  });

  eventSource.addEventListener('approval_needed', (e) => {
    const data = JSON.parse(e.data);
    showApproval(data);
  });

  eventSource.addEventListener('error', (e) => {
    if (e.data) {
      const data = JSON.parse(e.data);
      addMessage('system', 'Error: ' + data.message);
    }
  });
}

// --- Chat ---

function sendMessage() {
  const input = document.getElementById('chat-input');
  const content = input.value.trim();
  if (!content) return;

  addMessage('user', content);
  input.value = '';
  autoResizeTextarea(input);
  setStatus('Sending...', true);

  const body = { content };
  if (currentThreadId) body.thread_id = currentThreadId;

  apiFetch('/api/chat/send', {
    method: 'POST',
    body: body,
  }).catch((err) => {
    addMessage('system', 'Failed to send: ' + err.message);
    setStatus('');
  });
}

function sendApprovalAction(requestId, action) {
  apiFetch('/api/chat/approval', {
    method: 'POST',
    body: { request_id: requestId, action: action },
  }).catch((err) => {
    addMessage('system', 'Failed to send approval: ' + err.message);
  });

  // Disable buttons and show confirmation on the card
  const card = document.querySelector('.approval-card[data-request-id="' + requestId + '"]');
  if (card) {
    const buttons = card.querySelectorAll('.approval-actions button');
    buttons.forEach((btn) => {
      btn.disabled = true;
    });
    const actions = card.querySelector('.approval-actions');
    const label = document.createElement('span');
    label.className = 'approval-resolved';
    const labelText = action === 'approve' ? 'Approved' : action === 'always' ? 'Always approved' : 'Denied';
    label.textContent = labelText;
    actions.appendChild(label);
  }
}

function renderMarkdown(text) {
  if (typeof marked !== 'undefined') {
    return marked.parse(text);
  }
  return escapeHtml(text);
}

function addMessage(role, content) {
  const container = document.getElementById('chat-messages');
  const div = document.createElement('div');
  div.className = 'message ' + role;
  if (role === 'user') {
    div.textContent = content;
  } else {
    div.setAttribute('data-raw', content);
    div.innerHTML = renderMarkdown(content);
  }
  container.appendChild(div);
  container.scrollTop = container.scrollHeight;
}

function appendToLastAssistant(chunk) {
  const container = document.getElementById('chat-messages');
  const messages = container.querySelectorAll('.message.assistant');
  if (messages.length > 0) {
    const last = messages[messages.length - 1];
    const raw = (last.getAttribute('data-raw') || '') + chunk;
    last.setAttribute('data-raw', raw);
    last.innerHTML = renderMarkdown(raw);
    container.scrollTop = container.scrollHeight;
  } else {
    addMessage('assistant', chunk);
  }
}

function setStatus(text, spinning) {
  const el = document.getElementById('chat-status');
  if (!text) {
    el.innerHTML = '';
    return;
  }

  // Thinking dots animation
  if (spinning) {
    const prefixes = ['Thinking', 'Processing', 'Calculating', 'Burunyuu', 'Meowing', 'Dreaming'];
    const suffixes = ['nya', 'purr', 'mrrp', 'burunyuu', '✨', '🐾'];
    const randomPrefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    const randomSuffix = suffixes[Math.floor(Math.random() * suffixes.length)];

    // Strip trailing dots from text since we're adding animated ones
    const cleanText = text.replace(/\.+$/, '');
    const displayMsg = spinning ? `${randomPrefix} ${randomSuffix}` : cleanText;

    el.innerHTML = '<div class="spinner"></div>' + escapeHtml(displayMsg) + '<span class="thinking-dots"><span>.</span><span>.</span><span>.</span></span>';
  } else {
    el.textContent = text;
  }
}

function processGlyphwave(text) {
  // Wrap purr-related words in glyphwave span
  const patterns = [
    /(\bpurr+\b)/gi,
    /(\bmrrp+\b)/gi,
    /(\bmeow+\b)/gi,
    /(\bburunyuu+\b)/gi,
    /(\bnyan+\b)/gi,
    /(\bnyan+\b)/gi
  ];

  let processed = text;
  for (const pattern of patterns) {
    processed = processed.replace(pattern, '<span class="glyphwave">$1</span>');
  }
  return processed;
}

function renderMarkdown(text) {
  let html = '';
  if (typeof marked !== 'undefined') {
    html = marked.parse(text);
  } else {
    html = escapeHtml(text);
  }
  return processGlyphwave(html);
}

function showApproval(data) {
  const container = document.getElementById('chat-messages');
  const card = document.createElement('div');
  card.className = 'approval-card';
  card.setAttribute('data-request-id', data.request_id);

  const header = document.createElement('div');
  header.className = 'approval-header';
  header.textContent = 'Tool requires approval';
  card.appendChild(header);

  const toolName = document.createElement('div');
  toolName.className = 'approval-tool-name';
  toolName.textContent = data.tool_name;
  card.appendChild(toolName);

  if (data.description) {
    const desc = document.createElement('div');
    desc.className = 'approval-description';
    desc.textContent = data.description;
    card.appendChild(desc);
  }

  if (data.parameters) {
    const paramsToggle = document.createElement('button');
    paramsToggle.className = 'approval-params-toggle';
    paramsToggle.textContent = 'Show parameters';
    const paramsBlock = document.createElement('pre');
    paramsBlock.className = 'approval-params';
    paramsBlock.textContent = data.parameters;
    paramsBlock.style.display = 'none';
    paramsToggle.addEventListener('click', () => {
      const visible = paramsBlock.style.display !== 'none';
      paramsBlock.style.display = visible ? 'none' : 'block';
      paramsToggle.textContent = visible ? 'Show parameters' : 'Hide parameters';
    });
    card.appendChild(paramsToggle);
    card.appendChild(paramsBlock);
  }

  const actions = document.createElement('div');
  actions.className = 'approval-actions';

  const approveBtn = document.createElement('button');
  approveBtn.className = 'approve';
  approveBtn.textContent = 'Approve';
  approveBtn.addEventListener('click', () => sendApprovalAction(data.request_id, 'approve'));

  const alwaysBtn = document.createElement('button');
  alwaysBtn.className = 'always';
  alwaysBtn.textContent = 'Always';
  alwaysBtn.addEventListener('click', () => sendApprovalAction(data.request_id, 'always'));

  const denyBtn = document.createElement('button');
  denyBtn.className = 'deny';
  denyBtn.textContent = 'Deny';
  denyBtn.addEventListener('click', () => sendApprovalAction(data.request_id, 'deny'));

  actions.appendChild(approveBtn);
  actions.appendChild(alwaysBtn);
  actions.appendChild(denyBtn);
  card.appendChild(actions);

  container.appendChild(card);
  container.scrollTop = container.scrollHeight;
}

function loadHistory() {
  let path = '/api/chat/history';
  if (currentThreadId) {
    path += '?thread_id=' + encodeURIComponent(currentThreadId);
  }

  apiFetch(path).then((data) => {
    const container = document.getElementById('chat-messages');
    container.innerHTML = '';
    for (const turn of data.turns) {
      addMessage('user', turn.user_input);
      if (turn.response) {
        addMessage('assistant', turn.response);
      }
    }
  }).catch(() => {
    // No history or no active thread, that's fine
  });
}

// Chat input auto-resize and keyboard handling
const chatInput = document.getElementById('chat-input');
chatInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    sendMessage();
  }
});
chatInput.addEventListener('input', () => autoResizeTextarea(chatInput));

function autoResizeTextarea(el) {
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 120) + 'px';
}

// --- Tabs ---

document.querySelectorAll('.tab-bar button[data-tab]').forEach((btn) => {
  btn.addEventListener('click', () => {
    const tab = btn.getAttribute('data-tab');
    switchTab(tab);
  });
});

function switchTab(tab) {
  currentTab = tab;
  document.querySelectorAll('.tab-bar button[data-tab]').forEach((b) => {
    b.classList.toggle('active', b.getAttribute('data-tab') === tab);
  });
  document.querySelectorAll('.tab-panel').forEach((p) => {
    p.classList.toggle('active', p.id === 'tab-' + tab);
  });

  if (tab === 'memory') loadMemoryTree();
  if (tab === 'jobs') loadJobs();
  if (tab === 'extensions') loadExtensions();
}

// --- Memory (filesystem tree) ---

let memorySearchTimeout = null;
// Tree state: nested nodes persisted across renders
// { name, path, is_dir, children: [] | null, expanded: bool, loaded: bool }
let memoryTreeState = null;

document.getElementById('memory-search').addEventListener('input', (e) => {
  clearTimeout(memorySearchTimeout);
  const query = e.target.value.trim();
  if (!query) {
    loadMemoryTree();
    return;
  }
  memorySearchTimeout = setTimeout(() => searchMemory(query), 300);
});

function loadMemoryTree() {
  // Only load top-level on first load (or refresh)
  apiFetch('/api/memory/list?path=').then((data) => {
    memoryTreeState = data.entries.map((e) => ({
      name: e.name,
      path: e.path,
      is_dir: e.is_dir,
      children: e.is_dir ? null : undefined,
      expanded: false,
      loaded: false,
    }));
    renderTree();
  }).catch(() => { });
}

function renderTree() {
  const container = document.getElementById('memory-tree');
  container.innerHTML = '';
  if (!memoryTreeState || memoryTreeState.length === 0) {
    container.innerHTML = '<div class="tree-item" style="color:var(--text-secondary)">No files in workspace</div>';
    return;
  }
  renderNodes(memoryTreeState, container, 0);
}

function renderNodes(nodes, container, depth) {
  for (const node of nodes) {
    const row = document.createElement('div');
    row.className = 'tree-row';
    row.style.paddingLeft = (depth * 16 + 8) + 'px';

    if (node.is_dir) {
      const arrow = document.createElement('span');
      arrow.className = 'expand-arrow' + (node.expanded ? ' expanded' : '');
      arrow.textContent = '\u25B6';
      arrow.addEventListener('click', (e) => {
        e.stopPropagation();
        toggleExpand(node);
      });
      row.appendChild(arrow);

      const label = document.createElement('span');
      label.className = 'tree-label dir';
      label.textContent = node.name;
      label.addEventListener('click', () => toggleExpand(node));
      row.appendChild(label);
    } else {
      const spacer = document.createElement('span');
      spacer.className = 'expand-arrow-spacer';
      row.appendChild(spacer);

      const label = document.createElement('span');
      label.className = 'tree-label file';
      label.textContent = node.name;
      label.addEventListener('click', () => readMemoryFile(node.path));
      row.appendChild(label);
    }

    container.appendChild(row);

    if (node.is_dir && node.expanded && node.children) {
      const childContainer = document.createElement('div');
      childContainer.className = 'tree-children';
      renderNodes(node.children, childContainer, depth + 1);
      container.appendChild(childContainer);
    }
  }
}

function toggleExpand(node) {
  if (node.expanded) {
    node.expanded = false;
    renderTree();
    return;
  }

  if (node.loaded) {
    node.expanded = true;
    renderTree();
    return;
  }

  // Lazy-load children
  apiFetch('/api/memory/list?path=' + encodeURIComponent(node.path)).then((data) => {
    node.children = data.entries.map((e) => ({
      name: e.name,
      path: e.path,
      is_dir: e.is_dir,
      children: e.is_dir ? null : undefined,
      expanded: false,
      loaded: false,
    }));
    node.loaded = true;
    node.expanded = true;
    renderTree();
  }).catch(() => { });
}

function readMemoryFile(path) {
  // Update breadcrumb
  document.getElementById('memory-breadcrumb').innerHTML = buildBreadcrumb(path);

  apiFetch('/api/memory/read?path=' + encodeURIComponent(path)).then((data) => {
    document.getElementById('memory-viewer').textContent = data.content;
  }).catch((err) => {
    document.getElementById('memory-viewer').innerHTML = '<div class="empty">Error: ' + escapeHtml(err.message) + '</div>';
  });
}

function buildBreadcrumb(path) {
  const parts = path.split('/');
  let html = '<a onclick="loadMemoryTree()">workspace</a>';
  let current = '';
  for (const part of parts) {
    current += (current ? '/' : '') + part;
    html += ' / <a onclick="readMemoryFile(\'' + escapeHtml(current) + '\')">' + escapeHtml(part) + '</a>';
  }
  return html;
}

function searchMemory(query) {
  apiFetch('/api/memory/search', {
    method: 'POST',
    body: { query, limit: 20 },
  }).then((data) => {
    const tree = document.getElementById('memory-tree');
    tree.innerHTML = '';
    if (data.results.length === 0) {
      tree.innerHTML = '<div class="tree-item" style="color:var(--text-secondary)">No results</div>';
      return;
    }
    for (const result of data.results) {
      const item = document.createElement('div');
      item.className = 'search-result';
      item.innerHTML = '<div class="path">' + escapeHtml(result.path) + '</div>'
        + '<div class="snippet">' + escapeHtml(result.content.substring(0, 120)) + '</div>';
      item.addEventListener('click', () => readMemoryFile(result.path));
      tree.appendChild(item);
    }
  }).catch(() => { });
}

// --- Logs ---

const LOG_MAX_ENTRIES = 2000;
let logsPaused = false;
let logBuffer = []; // buffer while paused

function connectLogSSE() {
  if (logEventSource) logEventSource.close();

  logEventSource = new EventSource('/api/logs/events?token=' + encodeURIComponent(token));

  logEventSource.addEventListener('log', (e) => {
    const entry = JSON.parse(e.data);
    if (logsPaused) {
      logBuffer.push(entry);
      return;
    }
    appendLogEntry(entry);
  });

  logEventSource.onerror = () => {
    // Silent reconnect
  };
}

function appendLogEntry(entry) {
  const output = document.getElementById('logs-output');

  // Level filter
  const levelFilter = document.getElementById('logs-level-filter').value;
  const targetFilter = document.getElementById('logs-target-filter').value.trim().toLowerCase();

  const div = document.createElement('div');
  div.className = 'log-entry level-' + entry.level;
  div.setAttribute('data-level', entry.level);
  div.setAttribute('data-target', entry.target);

  const ts = document.createElement('span');
  ts.className = 'log-ts';
  ts.textContent = entry.timestamp.substring(11, 23);
  div.appendChild(ts);

  const lvl = document.createElement('span');
  lvl.className = 'log-level';
  lvl.textContent = entry.level.padEnd(5);
  div.appendChild(lvl);

  const tgt = document.createElement('span');
  tgt.className = 'log-target';
  tgt.textContent = entry.target;
  div.appendChild(tgt);

  const msg = document.createElement('span');
  msg.className = 'log-msg';
  msg.textContent = entry.message;
  div.appendChild(msg);

  div.addEventListener('click', () => div.classList.toggle('expanded'));

  // Apply current filters as visibility
  const matchesLevel = levelFilter === 'all' || entry.level === levelFilter;
  const matchesTarget = !targetFilter || entry.target.toLowerCase().includes(targetFilter);
  if (!matchesLevel || !matchesTarget) {
    div.style.display = 'none';
  }

  output.appendChild(div);

  // Cap entries
  while (output.children.length > LOG_MAX_ENTRIES) {
    output.removeChild(output.firstChild);
  }

  // Auto-scroll
  if (document.getElementById('logs-autoscroll').checked) {
    output.scrollTop = output.scrollHeight;
  }
}

function toggleLogsPause() {
  logsPaused = !logsPaused;
  const btn = document.getElementById('logs-pause-btn');
  btn.textContent = logsPaused ? 'Resume' : 'Pause';

  if (!logsPaused) {
    // Flush buffer
    for (const entry of logBuffer) {
      appendLogEntry(entry);
    }
    logBuffer = [];
  }
}

function clearLogs() {
  document.getElementById('logs-output').innerHTML = '';
  logBuffer = [];
}

// Re-apply filters when level or target changes
document.getElementById('logs-level-filter').addEventListener('change', applyLogFilters);
document.getElementById('logs-target-filter').addEventListener('input', applyLogFilters);

function applyLogFilters() {
  const levelFilter = document.getElementById('logs-level-filter').value;
  const targetFilter = document.getElementById('logs-target-filter').value.trim().toLowerCase();
  const entries = document.querySelectorAll('#logs-output .log-entry');
  for (const el of entries) {
    const matchesLevel = levelFilter === 'all' || el.getAttribute('data-level') === levelFilter;
    const matchesTarget = !targetFilter || el.getAttribute('data-target').toLowerCase().includes(targetFilter);
    el.style.display = (matchesLevel && matchesTarget) ? '' : 'none';
  }
}

// --- Extensions ---

function loadExtensions() {
  const extList = document.getElementById('extensions-list');
  const toolsTbody = document.getElementById('tools-tbody');
  const toolsEmpty = document.getElementById('tools-empty');

  // Fetch both in parallel
  Promise.all([
    apiFetch('/api/extensions').catch(() => ({ extensions: [] })),
    apiFetch('/api/extensions/tools').catch(() => ({ tools: [] })),
  ]).then(([extData, toolData]) => {
    // Render extensions
    if (extData.extensions.length === 0) {
      extList.innerHTML = '<div class="empty-state">No extensions installed</div>';
    } else {
      extList.innerHTML = '';
      for (const ext of extData.extensions) {
        extList.appendChild(renderExtensionCard(ext));
      }
    }

    // Render tools
    if (toolData.tools.length === 0) {
      toolsTbody.innerHTML = '';
      toolsEmpty.style.display = 'block';
    } else {
      toolsEmpty.style.display = 'none';
      toolsTbody.innerHTML = toolData.tools.map((t) =>
        '<tr><td>' + escapeHtml(t.name) + '</td><td>' + escapeHtml(t.description) + '</td></tr>'
      ).join('');
    }
  });
}

function renderExtensionCard(ext) {
  const card = document.createElement('div');
  card.className = 'ext-card';

  const header = document.createElement('div');
  header.className = 'ext-header';

  const name = document.createElement('span');
  name.className = 'ext-name';
  name.textContent = ext.name;
  header.appendChild(name);

  const kind = document.createElement('span');
  kind.className = 'ext-kind kind-' + ext.kind;
  kind.textContent = ext.kind;
  header.appendChild(kind);

  const authDot = document.createElement('span');
  authDot.className = 'ext-auth-dot ' + (ext.authenticated ? 'authed' : 'unauthed');
  authDot.title = ext.authenticated ? 'Authenticated' : 'Not authenticated';
  header.appendChild(authDot);

  card.appendChild(header);

  if (ext.description) {
    const desc = document.createElement('div');
    desc.className = 'ext-desc';
    desc.textContent = ext.description;
    card.appendChild(desc);
  }

  if (ext.url) {
    const url = document.createElement('div');
    url.className = 'ext-url';
    url.textContent = ext.url;
    url.title = ext.url;
    card.appendChild(url);
  }

  if (ext.tools.length > 0) {
    const tools = document.createElement('div');
    tools.className = 'ext-tools';
    tools.textContent = 'Tools: ' + ext.tools.join(', ');
    card.appendChild(tools);
  }

  const actions = document.createElement('div');
  actions.className = 'ext-actions';

  if (!ext.active) {
    const activateBtn = document.createElement('button');
    activateBtn.className = 'btn-ext activate';
    activateBtn.textContent = 'Activate';
    activateBtn.addEventListener('click', () => activateExtension(ext.name));
    actions.appendChild(activateBtn);
  } else {
    const activeLabel = document.createElement('span');
    activeLabel.className = 'ext-active-label';
    activeLabel.textContent = 'Active';
    actions.appendChild(activeLabel);
  }

  const removeBtn = document.createElement('button');
  removeBtn.className = 'btn-ext remove';
  removeBtn.textContent = 'Remove';
  removeBtn.addEventListener('click', () => removeExtension(ext.name));
  actions.appendChild(removeBtn);

  card.appendChild(actions);
  return card;
}

function activateExtension(name) {
  apiFetch('/api/extensions/' + encodeURIComponent(name) + '/activate', { method: 'POST' })
    .then((res) => {
      if (res.success) {
        loadExtensions();
        return;
      }

      if (res.auth_url) {
        addMessage(
          'system',
          'Opening authentication for **' + name + '**. Complete the flow in the opened tab, then click Activate again.'
        );
        window.open(res.auth_url, '_blank');
      } else if (res.awaiting_token) {
        addMessage(
          'system',
          (res.instructions || 'Please provide an API token for **' + name + '**.') +
          '\n\nYou can authenticate via chat: type `Authenticate ' + name + '` and follow the instructions.'
        );
      } else {
        addMessage('system', 'Activate failed: ' + res.message);
      }
      loadExtensions();
    })
    .catch((err) => addMessage('system', 'Activate failed: ' + err.message));
}

function removeExtension(name) {
  apiFetch('/api/extensions/' + encodeURIComponent(name) + '/remove', { method: 'POST' })
    .then((res) => {
      if (!res.success) {
        addMessage('system', 'Remove failed: ' + res.message);
      }
      loadExtensions();
    })
    .catch((err) => addMessage('system', 'Remove failed: ' + err.message));
}

// --- Jobs ---

function loadJobs() {
  Promise.all([
    apiFetch('/api/jobs/summary'),
    apiFetch('/api/jobs'),
  ]).then(([summary, jobList]) => {
    renderJobsSummary(summary);
    renderJobsList(jobList.jobs);
  }).catch(() => { });
}

function renderJobsSummary(s) {
  document.getElementById('jobs-summary').innerHTML = ''
    + summaryCard('Total', s.total, '')
    + summaryCard('In Progress', s.in_progress, 'active')
    + summaryCard('Completed', s.completed, 'completed')
    + summaryCard('Failed', s.failed, 'failed')
    + summaryCard('Stuck', s.stuck, 'stuck');
}

function summaryCard(label, count, cls) {
  return '<div class="summary-card ' + cls + '">'
    + '<div class="count">' + count + '</div>'
    + '<div class="label">' + label + '</div>'
    + '</div>';
}

function renderJobsList(jobs) {
  const tbody = document.getElementById('jobs-tbody');
  const empty = document.getElementById('jobs-empty');

  if (jobs.length === 0) {
    tbody.innerHTML = '';
    empty.style.display = 'block';
    return;
  }

  empty.style.display = 'none';
  tbody.innerHTML = jobs.map((job) => {
    const shortId = job.id.substring(0, 8);
    const stateClass = job.state.replace(' ', '_');
    const cancelBtn = (job.state === 'pending' || job.state === 'in_progress')
      ? '<button class="btn-cancel" onclick="cancelJob(\'' + job.id + '\')">Cancel</button>'
      : '';
    return '<tr>'
      + '<td title="' + escapeHtml(job.id) + '">' + shortId + '</td>'
      + '<td>' + escapeHtml(job.title) + '</td>'
      + '<td><span class="badge ' + stateClass + '">' + escapeHtml(job.state) + '</span></td>'
      + '<td>' + formatDate(job.created_at) + '</td>'
      + '<td>' + cancelBtn + '</td>'
      + '</tr>';
  }).join('');
}

function cancelJob(jobId) {
  apiFetch('/api/jobs/' + jobId + '/cancel', { method: 'POST' })
    .then(() => loadJobs())
    .catch((err) => {
      addMessage('system', 'Failed to cancel job: ' + err.message);
    });
}

// --- Utilities ---

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function formatDate(isoString) {
  if (!isoString) return '-';
  const d = new Date(isoString);
  return d.toLocaleString();
}

// --- Threads ---

let currentThreadId = null;

function isCurrentThread(threadId) {
  if (!threadId) return true;
  if (!currentThreadId) return true;
  return threadId === currentThreadId;
}

function loadThreads() {
  apiFetch('/api/chat/threads').then((data) => {
    if (data.assistant_thread) {
      const meta = document.getElementById('assistant-meta');
      if (data.assistant_thread.message_count) {
        meta.textContent = `${data.assistant_thread.message_count} messages`;
      } else {
        meta.textContent = '';
      }

      const assistantItem = document.getElementById('assistant-thread');
      if (!currentThreadId) {
        assistantItem.classList.add('active');
      } else {
        assistantItem.classList.remove('active');
      }
    }

    const listEl = document.getElementById('thread-list');
    listEl.innerHTML = '';

    if (data.threads && data.threads.length > 0) {
      data.threads.forEach((t) => {
        const item = document.createElement('div');
        item.className = 'thread-item';
        if (currentThreadId === t.id) {
          item.classList.add('active');
        }

        const titleSpan = document.createElement('span');
        titleSpan.textContent = t.title || `Thread ${t.id.substring(0, 8)}`;
        titleSpan.onclick = () => switchThread(t.id);
        item.appendChild(titleSpan);

        const deleteBtn = document.createElement('button');
        deleteBtn.className = 'delete-thread-btn';
        deleteBtn.innerHTML = '🗑️';
        deleteBtn.title = 'Delete thread';
        deleteBtn.onclick = (e) => {
          e.stopPropagation();
          deleteThread(t.id);
        };
        item.appendChild(deleteBtn);

        listEl.appendChild(item);
      });
    }
  }).catch((err) => {
    console.error('Failed to load threads:', err);
  });
}

function switchThread(threadId) {
  currentThreadId = threadId;
  hasMore = false;
  oldestTimestamp = null;
  document.getElementById('chat-messages').innerHTML = '';
  loadHistory();
  loadThreads();
}

function switchToAssistant() {
  switchThread(null);
}

function createNewThread() {
  apiFetch('/api/chat/thread/new', { method: 'POST' }).then((data) => {
    currentThreadId = data.id || null;
    document.getElementById('chat-messages').innerHTML = '';
    loadHistory();
    loadThreads();
  }).catch((err) => {
    console.error('Failed to create thread:', err);
  });
}

function deleteThread(threadId) {
  if (!confirm('Nyaaa?! Are you sure you want to delete this thread permanently? 😿')) return;

  apiFetch('/api/chat/thread/delete', {
    method: 'POST',
    body: { thread_id: threadId }
  }).then(() => {
    if (currentThreadId === threadId) {
      switchToAssistant();
    }
    loadThreads();
  }).catch((err) => {
    console.error('Failed to delete thread:', err);
    alert('Oops! Failed to delete thread. 🙀');
  });
}

function toggleThreadSidebar() {
  const sidebar = document.getElementById('thread-sidebar');
  const btn = document.getElementById('thread-toggle-btn');
  sidebar.classList.toggle('collapsed');
  btn.classList.toggle('collapsed');

  if (sidebar.classList.contains('collapsed')) {
    btn.innerHTML = '&raquo;';
  } else {
    btn.innerHTML = '&laquo;';
  }
}

document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
    e.preventDefault();
    createNewThread();
  }
});

// --- Routines ---

let currentRoutineId = null;

function loadRoutines() {
  currentRoutineId = null;

  document.getElementById('routines-table').style.display = 'table';
  document.getElementById('routine-detail').style.display = 'none';

  apiFetch('/api/routines').then((data) => {
    renderRoutinesSummary(data.summary || {});

    if (data.routines && data.routines.length > 0) {
      renderRoutinesList(data.routines);
      document.getElementById('routines-table').style.display = 'table';
      document.getElementById('routines-empty').style.display = 'none';
    } else {
      document.getElementById('routines-table').style.display = 'none';
      document.getElementById('routines-empty').style.display = 'block';
    }
  }).catch((err) => {
    console.error('Failed to load routines:', err);
    document.getElementById('routines-empty').style.display = 'block';
  });
}

function renderRoutinesSummary(s) {
  document.getElementById('routines-summary').innerHTML = ''
    + summaryCard('Total', s.total, '')
    + summaryCard('Enabled', s.enabled, 'active')
    + summaryCard('Disabled', s.disabled, '');
}

function summaryCard(title, value, cls) {
  return `
    <div class="summary-card ${cls}">
      <div class="summary-card-title">${title}</div>
      <div class="summary-card-value">${value || 0}</div>
    </div>
  `;
}

function renderRoutinesList(routines) {
  const tbody = document.getElementById('routines-tbody');
  tbody.innerHTML = '';
  routines.forEach((r) => {
    const tr = document.createElement('tr');
    tr.onclick = () => openRoutineDetail(r.id);
    tr.innerHTML = `
      <td class="routine-name">${escapeHtml(r.name)}</td>
      <td>${escapeHtml(r.trigger_type || 'Manual')}</td>
      <td>${formatDate(r.last_run)}</td>
      <td>${formatDate(r.next_run)}</td>
      <td><span class="routine-status ${r.enabled ? 'active' : 'inactive'}">${r.enabled ? 'Active' : 'Inactive'}</span></td>
      <td class="routine-actions">
        <button onclick="event.stopPropagation(); triggerRoutine('${r.id}')">Run</button>
        <button onclick="event.stopPropagation(); toggleRoutine('${r.id}')">Toggle</button>
        <button class="danger" onclick="event.stopPropagation(); deleteRoutine('${r.id}', '${escapeHtml(r.name)}')">Delete</button>
      </td>
    `;
    tbody.appendChild(tr);
  });
}

function openRoutineDetail(id) {
  currentRoutineId = id;
  apiFetch('/api/routines/' + id).then((routine) => {
    renderRoutineDetail(routine);
  }).catch((err) => {
    console.error('Failed to load routine:', err);
  });
}

function closeRoutineDetail() {
  currentRoutineId = null;
  loadRoutines();
}

function renderRoutineDetail(routine) {
  const table = document.getElementById('routines-table');
  if (table) table.style.display = 'none';
  document.getElementById('routines-empty').style.display = 'none';

  const detail = document.getElementById('routine-detail');
  detail.style.display = 'block';
  detail.innerHTML = `
    <div style="margin-bottom: 20px">
      <button onclick="closeRoutineDetail()">&larr; Back to List</button>
    </div>
    <div class="routine-detail-content">
      <h2>${escapeHtml(routine.name)}</h2>
      <p><strong>ID:</strong> ${routine.id}</p>
      <p><strong>Trigger:</strong> ${routine.trigger_type || 'Manual'}</p>
      <p><strong>Status:</strong> <span class="routine-status ${routine.enabled ? 'active' : 'inactive'}">${routine.enabled ? 'Active' : 'Inactive'}</span></p>
      <p><strong>Last Run:</strong> ${formatDate(routine.last_run)}</p>
      <p><strong>Next Run:</strong> ${formatDate(routine.next_run)}</p>
      <p><strong>Description:</strong> ${escapeHtml(routine.description || 'No description')}</p>
      <div class="routine-actions" style="margin-top: 20px">
        <button onclick="triggerRoutine('${routine.id}')">Run Now</button>
        <button onclick="toggleRoutine('${routine.id}')">Toggle Enabled</button>
        <button class="danger" onclick="deleteRoutine('${routine.id}', '${escapeHtml(routine.name)}')">Delete</button>
      </div>
    </div>
  `;
}

function triggerRoutine(id) {
  apiFetch('/api/routines/' + id + '/trigger', { method: 'POST' })
    .then(() => showToast('Routine triggered', 'success'))
    .catch((err) => showToast('Trigger failed: ' + err.message, 'error'));
}

function toggleRoutine(id) {
  apiFetch('/api/routines/' + id + '/toggle', { method: 'POST' })
    .then((res) => {
      showToast('Routine ' + (res.status || 'toggled'), 'success');
      loadRoutines();
    })
    .catch((err) => showToast('Toggle failed: ' + err.message, 'error'));
}

function deleteRoutine(id, name) {
  if (!confirm('Delete routine "' + name + '"?')) return;
  apiFetch('/api/routines/' + id, { method: 'DELETE' })
    .then(() => {
      showToast('Routine deleted', 'success');
      loadRoutines();
    })
    .catch((err) => showToast('Delete failed: ' + err.message, 'error'));
}

function showToast(message, type) {
  console.log(`[${type}] ${message}`);
}

// --- Laboratory: Job Detail Modal ---

function showJobDetail(jobId) {
  const modal = document.getElementById('job-detail-modal');
  const title = document.getElementById('job-detail-title');
  const log = document.getElementById('job-activity-log');

  title.textContent = `Job ${jobId.substring(0, 8)} Activity`;
  log.innerHTML = '<div style="color: var(--text-dim)">Loading activity log...</div>';
  modal.style.display = 'flex';

  apiFetch('/api/jobs/' + jobId + '/activity').then((data) => {
    if (data.activity && data.activity.length > 0) {
      log.innerHTML = data.activity.map((entry) => {
        const cls = entry.level === 'error' ? 'error' : entry.level === 'success' ? 'success' : '';
        return `
          <div class="job-log-entry">
            <span class="job-log-ts">${formatDate(entry.timestamp)}</span>
            <span class="job-log-msg ${cls}">${escapeHtml(entry.message)}</span>
          </div>
        `;
      }).join('');
    } else {
      log.innerHTML = '<div style="color: var(--text-dim)">No activity logged yet.</div>';
    }
  }).catch((err) => {
    log.innerHTML = `<div style="color: var(--danger)">Failed to load activity: ${escapeHtml(err.message)}</div>`;
  });
}
