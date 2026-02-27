function copyToClipboard(elementId) {
  const el = document.getElementById(elementId);
  if (el) {
    navigator.clipboard.writeText(el.value || el.textContent);
  }
}

function filterPackages() {
  const input = document.getElementById('searchInput');
  if (!input) return;
  const filter = input.value.toLowerCase();
  const rows = document.querySelectorAll('.data-grid tbody tr');
  rows.forEach(row => {
    const name = (row.dataset.packageName || '').toLowerCase();
    const id = (row.dataset.packageId || '').toLowerCase();
    row.style.display = (!name.includes(filter) && !id.includes(filter)) ? 'none' : '';
  });
}

function timeago(ts) {
  var d = new Date(ts);
  var now = Date.now();
  var diff = Math.floor((now - d.getTime()) / 1000);
  if (diff < 60) return 'just now';
  if (diff < 3600) { var m = Math.floor(diff / 60); return m + (m === 1 ? ' minute' : ' minutes') + ' ago'; }
  if (diff < 86400) { var h = Math.floor(diff / 3600); return h + (h === 1 ? ' hour' : ' hours') + ' ago'; }
  if (diff < 2592000) { var dd = Math.floor(diff / 86400); return dd + (dd === 1 ? ' day' : ' days') + ' ago'; }
  if (diff < 31536000) { var mo = Math.floor(diff / 2592000); return mo + (mo === 1 ? ' month' : ' months') + ' ago'; }
  var y = Math.floor(diff / 31536000); return y + (y === 1 ? ' year' : ' years') + ' ago';
}

function formatAbsolute(ts) {
  var d = new Date(ts);
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
    + ' ' + d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
}

function initTimeago() {
  document.querySelectorAll('[data-timestamp]').forEach(function(el) {
    var ts = el.getAttribute('data-timestamp');
    el.textContent = timeago(ts);
    el.title = formatAbsolute(ts);
    el.style.cursor = 'pointer';
    el.addEventListener('click', function() {
      var showing = el.getAttribute('data-showing') || 'relative';
      if (showing === 'relative') {
        el.textContent = formatAbsolute(ts);
        el.title = timeago(ts);
        el.setAttribute('data-showing', 'absolute');
      } else {
        el.textContent = timeago(ts);
        el.title = formatAbsolute(ts);
        el.setAttribute('data-showing', 'relative');
      }
    });
  });
}

document.addEventListener('DOMContentLoaded', () => {
  const searchInput = document.getElementById('searchInput');
  if (searchInput) {
    searchInput.addEventListener('input', filterPackages);
  }

  document.querySelectorAll('[data-copy-target]').forEach(btn => {
    btn.addEventListener('click', () => {
      copyToClipboard(btn.dataset.copyTarget);
    });
  });

  document.querySelectorAll('[data-vcc-url]').forEach(btn => {
    btn.addEventListener('click', () => {
      window.location.href = 'vcc://vpm/addRepo?url=' + encodeURIComponent(btn.dataset.vccUrl);
    });
  });

  const helpBtn = document.getElementById('urlBarHelp');
  const helpDialog = document.getElementById('addListingToVccHelp');
  const helpClose = document.getElementById('addListingToVccHelpClose');
  if (helpBtn && helpDialog) {
    helpBtn.addEventListener('click', () => helpDialog.showModal());
  }
  if (helpClose && helpDialog) {
    helpClose.addEventListener('click', () => helpDialog.close());
  }

  initTimeago();
});
