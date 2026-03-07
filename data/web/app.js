// TizenClaw Dashboard — Vanilla JS SPA
(function () {
    'use strict';

    const API = '';  // Same origin

    // --- Navigation ---
    const navItems =
        document.querySelectorAll('.nav-item');
    const pages =
        document.querySelectorAll('.page');

    function navigateTo(page) {
        navItems.forEach(n =>
            n.classList.remove('active'));
        pages.forEach(p =>
            p.classList.remove('active'));

        const navEl =
            document.getElementById('nav-' + page);
        const pageEl =
            document.getElementById('page-' + page);
        if (navEl) navEl.classList.add('active');
        if (pageEl) pageEl.classList.add('active');

        // Load data for the page
        if (page === 'dashboard') loadDashboard();
        else if (page === 'sessions') loadSessions();
        else if (page === 'tasks') loadTasks();
        else if (page === 'logs') loadLogs();
    }

    navItems.forEach(item => {
        item.addEventListener('click', () => {
            navigateTo(item.dataset.page);
        });
    });

    // --- API Helpers ---
    async function apiFetch(endpoint) {
        try {
            const res = await fetch(
                API + '/api/' + endpoint);
            return await res.json();
        } catch (e) {
            console.error('API error:', e);
            return null;
        }
    }

    async function apiPost(endpoint, body) {
        try {
            const res = await fetch(
                API + '/api/' + endpoint, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(body)
            });
            return await res.json();
        } catch (e) {
            console.error('API error:', e);
            return null;
        }
    }

    // --- Dashboard ---
    async function loadDashboard() {
        const status = await apiFetch('status');
        if (status) {
            document.getElementById('stat-status')
                .textContent = status.status || '—';
            document.getElementById('stat-version')
                .textContent = status.version || '—';
        }

        const sessions = await apiFetch('sessions');
        if (sessions) {
            document.getElementById('stat-sessions')
                .textContent = sessions.length;
        }

        const tasks = await apiFetch('tasks');
        if (tasks) {
            document.getElementById('stat-tasks')
                .textContent = tasks.length;
        }
    }

    // --- Sessions ---
    async function loadSessions() {
        const data = await apiFetch('sessions');
        const list =
            document.getElementById('session-list');

        if (!data || data.length === 0) {
            list.innerHTML =
                '<p class="empty-state">' +
                'No active sessions</p>';
            return;
        }

        list.innerHTML = data.map(s => {
            const sizeKB =
                (s.size_bytes / 1024).toFixed(1);
            const modified = s.modified ?
                new Date(s.modified * 1000)
                    .toLocaleString() : '—';
            return '<div class="card-item">' +
                '<div class="card-item-title">' +
                escHtml(s.id) + '</div>' +
                '<div class="card-item-meta">' +
                sizeKB + ' KB · ' +
                modified + '</div></div>';
        }).join('');
    }

    // --- Tasks ---
    async function loadTasks() {
        const data = await apiFetch('tasks');
        const list =
            document.getElementById('task-list');

        if (!data || data.length === 0) {
            list.innerHTML =
                '<p class="empty-state">' +
                'No scheduled tasks</p>';
            return;
        }

        list.innerHTML = data.map(t =>
            '<div class="card-item">' +
            '<div class="card-item-title">' +
            escHtml(t.file) + '</div>' +
            '<div class="card-item-meta">' +
            escHtml(t.content_preview || '') +
            '</div></div>'
        ).join('');
    }

    // --- Logs ---
    async function loadLogs() {
        const data = await apiFetch('logs');
        const logEl =
            document.getElementById('log-content');

        if (!data || data.length === 0) {
            logEl.textContent = 'No logs available.';
            return;
        }

        logEl.textContent =
            data.map(l => l.content).join('\n\n');
    }

    // --- Chat ---
    const chatInput =
        document.getElementById('chat-input');
    const chatSend =
        document.getElementById('chat-send');
    const chatMessages =
        document.getElementById('chat-messages');
    const chatSession =
        document.getElementById('chat-session');

    function addChatMsg(role, text) {
        // Remove welcome message if present
        const welcome =
            chatMessages.querySelector('.chat-welcome');
        if (welcome) welcome.remove();

        const el = document.createElement('div');
        el.className = 'chat-msg ' + role;
        el.textContent = text;
        chatMessages.appendChild(el);
        chatMessages.scrollTop =
            chatMessages.scrollHeight;
    }

    async function sendChat() {
        const prompt = chatInput.value.trim();
        if (!prompt) return;

        const sessionId = chatSession.value.trim() ||
            'web_dashboard';

        addChatMsg('user', prompt);
        chatInput.value = '';
        chatSend.disabled = true;

        const resp = await apiPost('chat', {
            prompt: prompt,
            session_id: sessionId
        });

        chatSend.disabled = false;

        if (resp && resp.response) {
            addChatMsg('assistant', resp.response);
        } else {
            addChatMsg('assistant',
                'Error: no response from agent.');
        }
    }

    chatSend.addEventListener('click', sendChat);
    chatInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendChat();
        }
    });

    // --- Utility ---
    function escHtml(s) {
        const div = document.createElement('div');
        div.textContent = s;
        return div.innerHTML;
    }

    // --- Initial Load ---
    loadDashboard();
})();
