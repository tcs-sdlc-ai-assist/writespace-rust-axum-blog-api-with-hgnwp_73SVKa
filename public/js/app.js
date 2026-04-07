// WriteSpace Frontend - Shared App Module
// Auth helpers, API integration, and UI utilities

(function (window) {
  "use strict";

  var TOKEN_KEY = "writespace_token";

  // ─── Token Management ───

  function getToken() {
    return localStorage.getItem(TOKEN_KEY);
  }

  function setToken(token) {
    localStorage.setItem(TOKEN_KEY, token);
  }

  function clearToken() {
    localStorage.removeItem(TOKEN_KEY);
  }

  function isLoggedIn() {
    var token = getToken();
    if (!token) return false;
    var user = getUser();
    if (!user) return false;
    if (user.exp && user.exp * 1000 < Date.now()) {
      clearToken();
      return false;
    }
    return true;
  }

  // ─── JWT Decoding ───

  function decodeBase64Url(str) {
    var base64 = str.replace(/-/g, "+").replace(/_/g, "/");
    var pad = base64.length % 4;
    if (pad) {
      base64 += new Array(5 - pad).join("=");
    }
    try {
      return decodeURIComponent(
        atob(base64)
          .split("")
          .map(function (c) {
            return "%" + ("00" + c.charCodeAt(0).toString(16)).slice(-2);
          })
          .join("")
      );
    } catch (e) {
      return null;
    }
  }

  function getUser() {
    var token = getToken();
    if (!token) return null;
    try {
      var parts = token.split(".");
      if (parts.length !== 3) return null;
      var payload = decodeBase64Url(parts[1]);
      if (!payload) return null;
      return JSON.parse(payload);
    } catch (e) {
      return null;
    }
  }

  // ─── API Fetch Wrapper ───

  function apiFetch(url, options) {
    options = options || {};
    options.headers = options.headers || {};

    var token = getToken();
    if (token) {
      options.headers["Authorization"] = "Bearer " + token;
    }

    if (
      options.body &&
      typeof options.body === "object" &&
      !(options.body instanceof FormData)
    ) {
      options.headers["Content-Type"] = "application/json";
      options.body = JSON.stringify(options.body);
    }

    if (!options.headers["Accept"]) {
      options.headers["Accept"] = "application/json";
    }

    return fetch(url, options).then(function (response) {
      if (response.status === 401) {
        clearToken();
        if (
          window.location.pathname !== "/login.html" &&
          window.location.pathname !== "/" &&
          window.location.pathname !== "/index.html"
        ) {
          window.location.href = "/login.html";
        }
        return Promise.reject({ status: 401, error: "Unauthorized" });
      }

      if (response.status === 204) {
        return { ok: true, status: 204, data: null };
      }

      return response
        .json()
        .then(function (data) {
          if (!response.ok) {
            return Promise.reject({
              status: response.status,
              error: data.error || "Request failed",
              data: data,
            });
          }
          return { ok: true, status: response.status, data: data };
        })
        .catch(function (err) {
          if (err && err.status) {
            return Promise.reject(err);
          }
          if (!response.ok) {
            return Promise.reject({
              status: response.status,
              error: "Request failed",
            });
          }
          return { ok: true, status: response.status, data: null };
        });
    });
  }

  // ─── Auth Guards ───

  function requireAuth() {
    if (!isLoggedIn()) {
      window.location.href = "/login.html";
      return false;
    }
    return true;
  }

  function requireAdmin() {
    if (!isLoggedIn()) {
      window.location.href = "/login.html";
      return false;
    }
    var user = getUser();
    if (!user || user.role !== "admin") {
      window.location.href = "/blogs.html";
      return false;
    }
    return true;
  }

  // ─── Logout ───

  function logout() {
    clearToken();
    window.location.href = "/login.html";
  }

  // ─── UI Helpers ───

  function getRoleColor(role) {
    if (role === "admin") return "#e11d48";
    return "#6366f1";
  }

  function getRoleBadge(role) {
    var color = getRoleColor(role);
    return (
      '<span style="display:inline-block;padding:2px 8px;border-radius:9999px;font-size:0.7rem;font-weight:600;color:#fff;background:' +
      color +
      ';text-transform:uppercase;">' +
      escapeHtml(role) +
      "</span>"
    );
  }

  function getAvatarInitials(displayName) {
    if (!displayName) return "?";
    var parts = displayName.trim().split(/\s+/);
    if (parts.length >= 2) {
      return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return displayName.substring(0, 2).toUpperCase();
  }

  function renderAvatar(displayName, role, size) {
    size = size || 36;
    var initials = getAvatarInitials(displayName);
    var bgColor = getRoleColor(role);
    var fontSize = Math.max(12, Math.floor(size * 0.4));
    return (
      '<span style="display:inline-flex;align-items:center;justify-content:center;width:' +
      size +
      "px;height:" +
      size +
      "px;border-radius:50%;background:" +
      bgColor +
      ";color:#fff;font-size:" +
      fontSize +
      'px;font-weight:700;flex-shrink:0;">' +
      escapeHtml(initials) +
      "</span>"
    );
  }

  function escapeHtml(str) {
    if (!str) return "";
    var div = document.createElement("div");
    div.appendChild(document.createTextNode(str));
    return div.innerHTML;
  }

  function formatDate(dateStr) {
    if (!dateStr) return "";
    try {
      var date = new Date(dateStr);
      return date.toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    } catch (e) {
      return dateStr;
    }
  }

  function formatDateTime(dateStr) {
    if (!dateStr) return "";
    try {
      var date = new Date(dateStr);
      return date.toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch (e) {
      return dateStr;
    }
  }

  function renderNav(activePage) {
    var nav = document.getElementById("main-nav");
    if (!nav) return;

    var loggedIn = isLoggedIn();
    var user = getUser();
    var isAdmin = user && user.role === "admin";

    var html = '<div class="nav-inner">';
    html += '<a href="/" class="nav-brand">WriteSpace</a>';
    html += '<div class="nav-links">';

    html +=
      '<a href="/blogs.html" class="nav-link' +
      (activePage === "blogs" ? " active" : "") +
      '">Blogs</a>';

    if (loggedIn) {
      html +=
        '<a href="/new-post.html" class="nav-link' +
        (activePage === "new-post" ? " active" : "") +
        '">New Post</a>';

      if (isAdmin) {
        html +=
          '<a href="/admin.html" class="nav-link' +
          (activePage === "admin" ? " active" : "") +
          '">Admin</a>';
      }

      html += '<div class="nav-user">';
      html += renderAvatar(user.display_name, user.role, 32);
      html += '<div class="nav-user-info">';
      html +=
        '<span class="nav-user-name">' +
        escapeHtml(user.display_name) +
        "</span>";
      html += getRoleBadge(user.role);
      html += "</div>";
      html +=
        '<button onclick="App.logout()" class="nav-logout-btn" title="Logout">';
      html +=
        '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>';
      html += "</button>";
      html += "</div>";
    } else {
      html +=
        '<a href="/login.html" class="nav-link' +
        (activePage === "login" ? " active" : "") +
        '">Login</a>';
      html +=
        '<a href="/register.html" class="nav-link' +
        (activePage === "register" ? " active" : "") +
        '">Register</a>';
    }

    html += "</div>";
    html += "</div>";

    nav.innerHTML = html;
  }

  function showToast(message, type) {
    type = type || "info";
    var existing = document.getElementById("app-toast");
    if (existing) {
      existing.remove();
    }

    var toast = document.createElement("div");
    toast.id = "app-toast";

    var bgColor = "#333";
    if (type === "success") bgColor = "#059669";
    if (type === "error") bgColor = "#dc2626";
    if (type === "warning") bgColor = "#d97706";

    toast.style.cssText =
      "position:fixed;top:20px;right:20px;padding:12px 24px;border-radius:8px;color:#fff;font-size:0.9rem;z-index:10000;box-shadow:0 4px 12px rgba(0,0,0,0.15);transition:opacity 0.3s ease;background:" +
      bgColor +
      ";";
    toast.textContent = message;

    document.body.appendChild(toast);

    setTimeout(function () {
      toast.style.opacity = "0";
      setTimeout(function () {
        if (toast.parentNode) {
          toast.parentNode.removeChild(toast);
        }
      }, 300);
    }, 3000);
  }

  function showLoading(container) {
    if (typeof container === "string") {
      container = document.getElementById(container);
    }
    if (!container) return;
    container.innerHTML =
      '<div style="display:flex;justify-content:center;align-items:center;padding:40px;">' +
      '<div style="width:32px;height:32px;border:3px solid #e5e7eb;border-top-color:#6366f1;border-radius:50%;animation:spin 0.8s linear infinite;"></div>' +
      "</div>";
  }

  function showError(container, message) {
    if (typeof container === "string") {
      container = document.getElementById(container);
    }
    if (!container) return;
    container.innerHTML =
      '<div style="text-align:center;padding:40px;color:#dc2626;">' +
      '<p style="font-size:1.1rem;font-weight:600;">Error</p>' +
      '<p style="margin-top:8px;color:#6b7280;">' +
      escapeHtml(message) +
      "</p>" +
      "</div>";
  }

  function showEmpty(container, message) {
    if (typeof container === "string") {
      container = document.getElementById(container);
    }
    if (!container) return;
    message = message || "Nothing to show.";
    container.innerHTML =
      '<div style="text-align:center;padding:40px;color:#9ca3af;">' +
      '<p style="font-size:1rem;">' +
      escapeHtml(message) +
      "</p>" +
      "</div>";
  }

  // ─── Inject global spinner keyframes ───

  (function injectStyles() {
    if (document.getElementById("app-global-styles")) return;
    var style = document.createElement("style");
    style.id = "app-global-styles";
    style.textContent = "@keyframes spin { to { transform: rotate(360deg); } }";
    document.head.appendChild(style);
  })();

  // ─── Public API ───

  window.App = {
    getToken: getToken,
    setToken: setToken,
    clearToken: clearToken,
    isLoggedIn: isLoggedIn,
    getUser: getUser,
    apiFetch: apiFetch,
    requireAuth: requireAuth,
    requireAdmin: requireAdmin,
    logout: logout,
    escapeHtml: escapeHtml,
    formatDate: formatDate,
    formatDateTime: formatDateTime,
    getRoleBadge: getRoleBadge,
    getRoleColor: getRoleColor,
    renderAvatar: renderAvatar,
    getAvatarInitials: getAvatarInitials,
    renderNav: renderNav,
    showToast: showToast,
    showLoading: showLoading,
    showError: showError,
    showEmpty: showEmpty,
  };
})(window);