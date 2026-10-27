(() => {
  // Debug-only smoke test, injected when the binary is launched with --self-test.
  // It verifies that the official frontend mounted the pet dock and that the
  // native bridge answers; it never writes account data.
  const run = async () => {
    const result = {};
    const invoke = window.__TAURI__.core.invoke;
    result.input = await invoke('get_input_status');
    result.safeArea = await invoke('get_main_desktop_safe_area');
    await new Promise((resolve, reject) => {
      const started = performance.now();
      const check = () => {
        const bounds = document.querySelector('.pet-dock')?.getBoundingClientRect();
        if (bounds && bounds.width > 0 && bounds.height > 0
          && bounds.left >= 0 && bounds.top >= 0
          && bounds.right <= innerWidth + 1 && bounds.bottom <= innerHeight + 1) return resolve();
        if (performance.now() - started > 15000) {
          return reject(new Error(`Pet layout not ready: ${JSON.stringify(bounds?.toJSON())}; viewport ${innerWidth}x${innerHeight}`));
        }
        requestAnimationFrame(check);
      };
      requestAnimationFrame(check);
    });
    const bounds = document.querySelector('.pet-dock').getBoundingClientRect();
    result.petVisible = bounds.width > 0 && bounds.height > 0
      && bounds.left >= 0 && bounds.top >= 0
      && bounds.right <= innerWidth && bounds.bottom <= innerHeight;
    const settingsButton = document.querySelector('.settings-btn');
    result.settingsButton = Boolean(settingsButton);
    if (settingsButton) {
      const rect = settingsButton.getBoundingClientRect();
      const view = (window.__TAURI__.webviewWindow ?? window.__TAURI__.window).getCurrentWebviewWindow?.()
        ?? window.__TAURI__.window.getCurrentWindow();
      const [scale, position] = await Promise.all([view.scaleFactor(), view.innerPosition()]);
      const payload = {
        x: position.x + (rect.left + rect.width / 2) * scale,
        y: position.y + (rect.top + rect.height / 2) * scale,
};
      await window.__TAURI__.event.emit('mouse-move', payload);
      await window.__TAURI__.event.emit('mouse-button', { button: 'left', action: 'press', memberId: null });
      await window.__TAURI__.event.emit('mouse-button', { button: 'left', action: 'release', memberId: null });
    }
    await new Promise(resolve => setTimeout(resolve, 1500));
    result.passed = result.petVisible;
    window.__WUWATAPPO_RUNTIME_SMOKE__ = result;
    await invoke('write_app_log', {
      level: result.passed ? 'info' : 'error',
      message: `[SELFTEST] ${JSON.stringify(result)}`,
    });
  };
  const start = () => run().catch(error => {
    window.__TAURI__.core.invoke('write_app_log', {
      level: 'error',
      message: `[SELFTEST] failed: ${String(error)}`,
    }).catch(console.error);
  });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start, { once: true });
  else start();
})();
