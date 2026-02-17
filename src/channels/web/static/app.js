// IronClaw Web Gateway - Client (Neco-Arc Edition) - Refined

let token = '';
let eventSource = null;
let logEventSource = null;
let currentTab = 'chat';
let currentThreadId = null;
let assistantThreadId = null;

// History state
let hasMore = false;
let oldestTimestamp = null;
let loadingOlder = false;

// Job state
let jobEvents = new Map(); // job_id -> Array of events
let jobListRefreshTimer = null;
const JOB_EVENTS_CAP = 500;
let currentJobId = null;
let currentJobSubTab = 'activity';
let jobFilesTreeState = null;

// Memory state
let memoryTreeState = null;
let currentMemoryPath = null;
let currentMemoryContent = null;
let memorySearchTimeout = null;

// Routine state
let currentRoutineId = null;

// Gateway status
let gatewayStatusInterval = null;

console.log("%c Burunyuu~ 😼 IronClaw Neco-Arc Edition (Refined) Loaded!", "color: #ffd700; font-size: 20px; background: #2b1d38; padding: 10px; border-radius: 10px;");

// --- Auth ---

function authenticate() {
  token = document.getElementById('token-input').value.trim();
  if (!token) {
    document.getElementById('auth-error').textContent = 'Token required';
    return;
  }

  apiFetch('/api/chat/threads')
    .then(() => {
      sessionStorage.setItem('ironclaw_token', token);
      document.getElementById('auth-screen').style.display = 'none';
      document.getElementById('app').style.display = 'flex';
      connectSSE();
      connectLogSSE();
      startGatewayStatusPolling();
      loadThreads();
      switchTab(currentTab);
    })
    .catch((err) => {
      console.error('Auth failed:', err);
      document.getElementById('auth-error').textContent = 'Invalid token (Burunyuu~ 😿)';
    });
}

document.getElementById('token-input').addEventListener('keydown', (e) => {
  if (e.key === 'Enter') authenticate();
});

(function autoAuth() {
  const saved = sessionStorage.getItem('ironclaw_token');
  if (saved) {
    document.getElementById('token-input').value = saved;
    authenticate();
  }
})();

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
    if (!isCurrentThread(data.thread_id)) return;
    addMessage('assistant', data.content);
    setStatus('');
    enableChatInput();
    loadThreads();
  });

  eventSource.addEventListener('thinking', (e) => {
    const data = JSON.parse(e.data);
    if (!isCurrentThread(data.thread_id)) return;
    setStatus(data.message, true);
  });

  eventSource.addEventListener('tool_started', (e) => {
    const data = JSON.parse(e.data);
    if (!isCurrentThread(data.thread_id)) return;
    setStatus('Running tool: ' + data.name, true);
  });

  eventSource.addEventListener('tool_completed', (e) => {
    const data = JSON.parse(e.data);
    if (!isCurrentThread(data.thread_id)) return;
    const icon = data.success ? '\u2713' : '\u2717';
    setStatus('Tool ' + data.name + ' ' + icon);
  });

  eventSource.addEventListener('stream_chunk', (e) => {
    const data = JSON.parse(e.data);
    if (!isCurrentThread(data.thread_id)) return;
    appendToLastAssistant(data.content);
  });

  eventSource.addEventListener('status', (e) => {
    const data = JSON.parse(e.data);
    if (!isCurrentThread(data.thread_id)) return;
    setStatus(data.message);
    if (data.message === 'Done' || data.message === 'Awaiting approval') {
      enableChatInput();
    }
  });

  eventSource.addEventListener('approval_needed', (e) => {
    const data = JSON.parse(e.data);
    showApproval(data);
  });

  eventSource.addEventListener('error', (e) => {
    if (e.data) {
      const data = JSON.parse(e.data);
      if (!isCurrentThread(data.thread_id)) return;
      addMessage('system', 'Error: ' + data.message);
      enableChatInput();
    }
  });

  // Job event listeners
  const jobEventTypes = ['job_message', 'job_tool_use', 'job_tool_result', 'job_status', 'job_result'];
  for (const evtType of jobEventTypes) {
    eventSource.addEventListener(evtType, (e) => {
      const data = JSON.parse(e.data);
      const jobId = data.job_id;
      if (!jobId) return;
      if (!jobEvents.has(jobId)) jobEvents.set(jobId, []);
      const events = jobEvents.get(jobId);
      events.push({ type: evtType, data: data, ts: Date.now() });
      while (events.length > JOB_EVENTS_CAP) events.shift();
      if (currentJobId === jobId && currentJobSubTab === 'activity') {
        refreshActivityTab(jobId);
      }
      if ((evtType === 'job_result' || evtType === 'job_status') && currentTab === 'jobs' && !currentJobId) {
        clearTimeout(jobListRefreshTimer);
        jobListRefreshTimer = setTimeout(loadJobs, 200);
      }
    });
  }
}

function isCurrentThread(threadId) {
  if (!threadId) return true;
  if (!currentThreadId) return true;
  return threadId === currentThreadId;
}

// --- Chat ---

function sendMessage() {
  const input = document.getElementById('chat-input');
  const sendBtn = document.getElementById('send-btn');
  const content = input.value.trim();
  if (!content) return;

  addMessage('user', content);
  input.value = '';
  autoResizeTextarea(input);
  setStatus('Sending...', true);

  sendBtn.disabled = true;
  input.disabled = true;

  apiFetch('/api/chat/send', {
    method: 'POST',
    body: { content, thread_id: currentThreadId || undefined },
  }).catch((err) => {
    addMessage('system', 'Failed to send: ' + err.message);
    setStatus('');
    enableChatInput();
  });
}

function enableChatInput() {
  const input = document.getElementById('chat-input');
  const sendBtn = document.getElementById('send-btn');
  if (sendBtn) sendBtn.disabled = false;
  if (input) {
    input.disabled = false;
    input.focus();
  }
}

function sendApprovalAction(requestId, action) {
  apiFetch('/api/chat/approval', {
    method: 'POST',
    body: { request_id: requestId, action: action, thread_id: currentThreadId },
  }).catch((err) => {
    addMessage('system', 'Failed to send approval: ' + err.message);
  });

  const card = document.querySelector('.approval-card[data-request-id="' + requestId + '"]');
  if (card) {
    const buttons = card.querySelectorAll('.approval-actions button');
    buttons.forEach((btn) => { btn.disabled = true; });
    const actions = card.querySelector('.approval-actions');
    const label = document.createElement('span');
    label.className = 'approval-resolved';
    const labelText = action === 'approve' ? 'Approved' : action === 'always' ? 'Always approved' : 'Denied';
    label.textContent = labelText;
    actions.appendChild(label);
  }
}

function processGlyphwave(text) {
  const patterns = [
    /(\bpurr+\b)/gi,
    /(\bmrrp+\b)/gi,
    /(\bmeow+\b)/gi,
    /(\bburunyuu+\b)/gi,
    /(\bnyan+\b)/gi,
    /(\bdoridoridori+\b)/gi
  ];
  let processed = text;
  for (const pattern of patterns) {
    processed = processed.replace(pattern, '<span class="glyphwave">$1</span>');
  }
  return processed;
}

function sanitizeRenderedHtml(html) {
  return html.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/\s+on\w+\s*=\s*["'][^"']*["']/gi, '');
}

function renderMarkdown(text) {
  if (typeof marked !== 'undefined') {
    let html = marked.parse(text);
    html = sanitizeRenderedHtml(html);
    return processGlyphwave(html);
  }
  return processGlyphwave(escapeHtml(text));
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

  if (spinning) {
    const prefixes = ['Thinking', 'Processing', 'Calculating', 'Burunyuu', 'Meowing', 'Dreaming'];
    const suffixes = ['nya', 'purr', 'mrrp', 'burunyuu', '✨', '🐾'];
    const randomPrefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    const randomSuffix = suffixes[Math.floor(Math.random() * suffixes.length)];
    el.innerHTML = '<div class="spinner"></div>' + escapeHtml(`${randomPrefix} ${randomSuffix}`) + '<span class="thinking-dots"><span>.</span><span>.</span><span>.</span></span>';
  } else {
    el.textContent = text;
  }
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

  const actions = document.createElement('div');
  actions.className = 'approval-actions';

  ['approve', 'always', 'deny'].forEach(action => {
    const btn = document.createElement('button');
    btn.className = action;
    btn.textContent = action.charAt(0).toUpperCase() + action.slice(1);
    btn.addEventListener('click', () => sendApprovalAction(data.request_id, action));
    actions.appendChild(btn);
  });

  card.appendChild(actions);
  container.appendChild(card);
  container.scrollTop = container.scrollHeight;
}

function loadHistory(before) {
  let url = '/api/chat/history?limit=50';
  if (currentThreadId) url += '&thread_id=' + encodeURIComponent(currentThreadId);
  if (before) url += '&before=' + encodeURIComponent(before);

  apiFetch(url).then((data) => {
    const container = document.getElementById('chat-messages');
    if (!before) container.innerHTML = '';

    for (const turn of data.turns) {
      addMessage('user', turn.user_input);
      if (turn.response) addMessage('assistant', turn.response);
    }
    container.scrollTop = container.scrollHeight;
    hasMore = data.has_more;
    oldestTimestamp = data.oldest_timestamp;
  }).catch(() => { });
}

// --- Threads ---

function loadThreads() {
  apiFetch('/api/chat/threads').then((data) => {
    if (data.assistant_thread) {
      assistantThreadId = data.assistant_thread.id;
      const el = document.getElementById('assistant-thread');
      el.className = 'assistant-item' + (currentThreadId === assistantThreadId ? ' active' : '');
      const meta = document.getElementById('assistant-meta');
      meta.textContent = (data.assistant_thread.turn_count || 0) + ' turns';
    }

    const list = document.getElementById('thread-list');
    list.innerHTML = '';
    const threads = data.threads || [];
    for (const t of threads) {
      const item = document.createElement('div');
      item.className = 'thread-item' + (t.id === currentThreadId ? ' active' : '');
      item.onclick = () => switchThread(t.id);

      const label = document.createElement('span');
      label.className = 'thread-label';
      label.textContent = t.title ? t.title : (t.id ? t.id.substring(0, 8) : 'New Thread');
      if (!label.textContent.trim()) label.textContent = 'Untitled Thread';
      // label.onclick removed, handled by item
      item.appendChild(label);

      const actions = document.createElement('div');
      actions.className = 'thread-actions';

      const renameBtn = document.createElement('button');
      renameBtn.className = 'rename-thread-btn';
      renameBtn.innerHTML = '✏️';
      renameBtn.onclick = (e) => { e.stopPropagation(); renameThread(t.id); };
      actions.appendChild(renameBtn);

      const deleteBtn = document.createElement('button');
      deleteBtn.className = 'delete-thread-btn';
      deleteBtn.innerHTML = '🗑️';
      deleteBtn.onclick = (e) => { e.stopPropagation(); deleteThread(t.id); };
      actions.appendChild(deleteBtn);

      item.appendChild(actions);
      list.appendChild(item);
    }

    if (!currentThreadId) {
      if (data.active_thread) {
        switchThread(data.active_thread);
      } else if (assistantThreadId) {
        switchToAssistant();
      } else if (threads.length > 0) {
        switchThread(threads[0].id);
      }
    }
  }).catch(() => { });
}

function switchToAssistant() {
  if (!assistantThreadId) return;
  switchThread(assistantThreadId);
}

function switchThread(threadId) {
  currentThreadId = threadId;
  document.getElementById('chat-messages').innerHTML = '';
  loadHistory();
  loadThreads();
}

function createNewThread() {
  apiFetch('/api/chat/thread/new', { method: 'POST' }).then((data) => {
    currentThreadId = data.id || null;
    document.getElementById('chat-messages').innerHTML = '';
    loadHistory();
    loadThreads();
  }).catch((err) => console.error(err));
}

function renameThread(threadId) {
  const newTitle = prompt('Enter new thread title: (Burunyuu~?)');
  if (!newTitle) return;

  apiFetch('/api/chat/thread/rename', {
    method: 'POST',
    body: { thread_id: threadId, title: newTitle }
  }).then(() => {
    loadThreads();
  }).catch((err) => {
    console.error('Failed to rename thread:', err);
    alert('Failed to rename thread. 😿');
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
    } else {
      // Add small delay to allow DB to settle before refreshing list
      setTimeout(loadThreads, 300);
    }
  }).catch((err) => console.error(err));
}

function toggleThreadSidebar() {
  const sidebar = document.getElementById('thread-sidebar');
  const btn = document.getElementById('thread-toggle-btn');
  sidebar.classList.toggle('collapsed');
  btn.innerHTML = sidebar.classList.contains('collapsed') ? '&raquo;' : '&laquo;';
}

// --- Memory (Filesystem) ---

function loadMemoryTree() {
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
    container.innerHTML = '<div class="empty">No files in workspace (Neco ate them?! 🥟)</div>';
    return;
  }
  renderNodes(memoryTreeState, container, 0);
}

function renderNodes(nodes, container, depth) {
  for (const node of nodes) {
    const item = document.createElement('div');
    item.className = 'memory-node' + (node.expanded ? ' expanded' : '');
    item.style.marginLeft = (depth * 20) + 'px';
    item.innerHTML = `<span>${node.is_dir ? (node.expanded ? '📂' : '📁') : '📄'} ${node.name}</span>`;
    item.onclick = (e) => {
      e.stopPropagation();
      if (node.is_dir) toggleExpand(node);
      else readMemoryFile(node.path);
    };
    container.appendChild(item);

    if (node.is_dir && node.expanded && node.children) {
      const childContainer = document.createElement('div');
      childContainer.className = 'memory-children';
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
  currentMemoryPath = path;
  document.getElementById('memory-breadcrumb-path').textContent = path;
  apiFetch('/api/memory/read?path=' + encodeURIComponent(path)).then((data) => {
    currentMemoryContent = data.content;
    const viewer = document.getElementById('memory-viewer');
    viewer.textContent = data.content;
  }).catch((err) => {
    document.getElementById('memory-viewer').textContent = 'Error: ' + err.message;
  });
}

// --- Jobs ---

function loadJobs() {
  Promise.all([
    apiFetch('/api/jobs/summary'),
    apiFetch('/api/jobs'),
  ]).then(([summary, listData]) => {
    renderJobsSummary(summary);
    renderJobsList(listData.jobs);
  }).catch(() => { });
}

function renderJobsSummary(s) {
  const container = document.getElementById('jobs-summary');
  if (!container) return;
  container.innerHTML = `
    <div class="summary-card"><span>Total</span><span>${s.total}</span></div>
    <div class="summary-card"><span>Active</span><span>${s.in_progress}</span></div>
    <div class="summary-card"><span>Done</span><span>${s.completed}</span></div>
    <div class="summary-card"><span>Failed</span><span>${s.failed}</span></div>
  `;
}

function renderJobsList(jobs) {
  const tbody = document.getElementById('jobs-tbody');
  if (!tbody) return;
  tbody.innerHTML = jobs.map(j => `
    <tr onclick="openJobDetail('${j.id}')">
      <td class="routine-name">${j.title || j.id.substring(0, 8)}</td>
      <td><span class="routine-status ${j.state === 'completed' ? 'active' : 'inactive'}">${j.state}</span></td>
      <td>${formatDate(j.created_at)}</td>
    </tr>
  `).join('');
}

function openJobDetail(jobId) {
  currentJobId = jobId;
  apiFetch('/api/jobs/' + jobId).then((job) => {
    // Basic detail view in modal
    alert(`Job Details (Burunyuu~):\nTitle: ${job.title}\nState: ${job.state}\nCreated: ${formatDate(job.created_at)}`);
  });
}

// --- Routines ---

function loadRoutines() {
  apiFetch('/api/routines').then((data) => {
    const tbody = document.getElementById('routines-tbody');
    if (!tbody) return;
    tbody.innerHTML = data.routines.map(r => `
      <tr>
        <td class="routine-name">${r.name}</td>
        <td><span class="routine-status ${r.enabled ? 'active' : 'inactive'}">${r.enabled ? 'Active' : 'Disabled'}</span></td>
        <td>${r.run_count}</td>
        <td>
          <button onclick="triggerRoutine('${r.id}')">Run</button>
        </td>
      </tr>
    `).join('');
  }).catch(() => { });
}

function triggerRoutine(id) {
  apiFetch('/api/routines/' + id + '/trigger', { method: 'POST' })
    .then(() => alert('Routine triggered! ✨'))
    .catch((err) => alert('Fail: ' + err.message));
}

// --- Tabs & Layout ---

function switchTab(tab) {
  currentTab = tab;
  document.querySelectorAll('.tab-bar button').forEach(b => b.classList.toggle('active', b.getAttribute('data-tab') === tab));
  document.querySelectorAll('.tab-panel').forEach(p => p.classList.toggle('active', p.id === 'tab-' + tab));

  if (tab === 'memory') loadMemoryTree();
  if (tab === 'jobs') loadJobs();
  if (tab === 'routines') loadRoutines();
  if (tab === 'extensions') loadExtensions();
  if (tab === 'logs') loadLogs();
}

// --- Logs ---

function loadLogs() {
  const filterLevel = document.getElementById('logs-level-filter').value;
  const filterTarget = document.getElementById('logs-target-filter').value;

  let url = '/api/logs?limit=100';
  if (filterLevel !== 'all') url += '&level=' + encodeURIComponent(filterLevel);
  if (filterTarget) url += '&target=' + encodeURIComponent(filterTarget);

  apiFetch(url).then((data) => {
    renderLogs(data.logs);
  }).catch((err) => {
    console.error('Failed to load logs:', err);
    document.getElementById('logs-output').innerHTML = '<div class="error">Failed to load logs: ' + err.message + '</div>';
  });
}

function renderLogs(logs) {
  const container = document.getElementById('logs-output');
  if (!container) return;

  if (!logs || logs.length === 0) {
    container.innerHTML = '<div class="empty-state">No logs found</div>';
    return;
  }

  container.innerHTML = logs.map(log => {
    const ts = new Date(log.timestamp).toLocaleTimeString();
    const levelClass = 'log-' + log.level.toLowerCase();
    return `<div class="log-entry ${levelClass}">
        <span class="log-ts">[${ts}]</span>
        <span class="log-level">${escapeHtml(log.level)}</span>
        <span class="log-target">${escapeHtml(log.target)}</span>
        <span class="log-msg">${escapeHtml(log.message)}</span>
      </div>`;
  }).join('');

  if (document.getElementById('logs-autoscroll').checked) {
    container.scrollTop = container.scrollHeight;
  }
}

function clearLogs() {
  document.getElementById('logs-output').innerHTML = '';
}

function toggleLogsPause() {
  // Implementation for pause if needed, or just toggle button text
  const btn = document.getElementById('logs-pause-btn');
  if (btn.textContent === 'Pause') {
    btn.textContent = 'Resume';
    // Logic to stop polling/SSE update for logs
  } else {
    btn.textContent = 'Pause';
    loadLogs();
  }
}

function startGatewayStatusPolling() {
  const poll = () => {
    apiFetch('/api/health').then(data => {
      document.getElementById('sse-status').textContent = 'Connected (Gateway OK)';
    }).catch(() => {
      document.getElementById('sse-status').textContent = 'Disconnected (Gateway Down!)';
    });
  };
  poll();
  gatewayStatusInterval = setInterval(poll, 30000);
}

function autoResizeTextarea(el) {
  el.style.height = 'auto';
  el.style.height = Math.min(el.scrollHeight, 120) + 'px';
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

function formatDate(iso) {
  return iso ? new Date(iso).toLocaleString() : '-';
}

// --- Extensions ---

function loadExtensions() {
  Promise.all([
    apiFetch('/api/extensions'),
    apiFetch('/api/extensions/tools'),
  ]).then(([extData, toolData]) => {
    renderExtensionsList(extData.extensions);
    renderToolsList(toolData.tools);
  }).catch((err) => {
    console.error('Failed to load extensions:', err);
    document.getElementById('extensions-list').innerHTML = '<div class="empty-state">Failed to load extensions</div>';
  });
}

function renderExtensionsList(extensions) {
  const container = document.getElementById('extensions-list');
  if (!container) return;

  if (!extensions || extensions.length === 0) {
    container.innerHTML = '<div class="empty-state">No extensions installed</div>';
    return;
  }

  container.innerHTML = extensions.map(ext => `
    <div class="extension-card">
      <div class="extension-header">
        <span class="extension-name">${escapeHtml(ext.name)}</span>
        <span class="extension-kind badge">${escapeHtml(ext.kind)}</span>
        <span class="extension-status ${ext.active ? 'active' : 'inactive'}">${ext.active ? 'Active' : 'Inactive'}</span>
      </div>
      <div class="extension-desc">${escapeHtml(ext.description || 'No description')}</div>
      <div class="extension-meta">
        ${ext.url ? `<span class="extension-url">${escapeHtml(ext.url)}</span>` : ''}
        ${ext.authenticated ? '<span class="extension-auth">Authenticated</span>' : ''}
      </div>
      <div class="extension-actions">
        ${!ext.active ? `<button onclick="activateExtension('${ext.name}')">Activate</button>` : ''}
        <button class="danger" onclick="removeExtension('${ext.name}')">Remove</button>
      </div>
    </div>
  `).join('');
}

function renderToolsList(tools) {
  const tbody = document.getElementById('tools-tbody');
  const empty = document.getElementById('tools-empty');
  if (!tbody) return;

  if (!tools || tools.length === 0) {
    tbody.innerHTML = '';
    if (empty) empty.style.display = 'block';
    return;
  }

  if (empty) empty.style.display = 'none';
  tbody.innerHTML = tools.map(tool => `
    <tr>
      <td class="tool-name">${escapeHtml(tool.name)}</td>
      <td class="tool-desc">${escapeHtml(tool.description || '')}</td>
    </tr>
  `).join('');
}

function activateExtension(name) {
  apiFetch('/api/extensions/' + encodeURIComponent(name) + '/activate', { method: 'POST' })
    .then((res) => {
      alert('Extension activated! 🚀');
      loadExtensions();
    })
    .catch(err => alert('Activation failed: ' + err.message));
}

function removeExtension(name) {
  if (!confirm('Are you sure you want to remove extension "' + name + '"?')) return;
  apiFetch('/api/extensions/' + encodeURIComponent(name) + '/remove', { method: 'POST' })
    .then(() => {
      alert('Extension removed.');
      loadExtensions();
    })
    .catch(err => alert('Removal failed: ' + err.message));
}

// --- Initialization ---


document.getElementById('send-btn').onclick = sendMessage;
document.getElementById('chat-input').onkeydown = (e) => {
  if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); sendMessage(); }
};

document.querySelectorAll('.tab-bar button[data-tab]').forEach((btn) => {
  btn.onclick = () => switchTab(btn.getAttribute('data-tab'));
});
