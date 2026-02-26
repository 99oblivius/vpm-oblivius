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
  const rows = document.querySelectorAll('#packageGrid fluent-data-grid-row:not([row-type="header"])');
  rows.forEach(row => {
    const name = (row.dataset.packageName || '').toLowerCase();
    const id = (row.dataset.packageId || '').toLowerCase();
    row.classList.toggle('hidden', !name.includes(filter) && !id.includes(filter));
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
    helpBtn.addEventListener('click', () => helpDialog.hidden = false);
  }
  if (helpClose && helpDialog) {
    helpClose.addEventListener('click', () => helpDialog.hidden = true);
  }
});
