import { ref, onMounted, onUnmounted } from "vue";
import { useVaultStore } from "../stores/vault";
import { useSettingsStore } from "../stores/settings";

const IDLE_CHECK_INTERVAL_MS = 5_000;
const BACKGROUND_GAP_SECONDS = 60;

/**
 * Auto-lock composable. Tracks user activity and locks the vault after a
 * configurable idle timeout. It also locks after extended WebView backgrounding
 * or a long wall-clock gap between timer ticks. These are defensive heuristics;
 * they are not presented as a guaranteed native OS suspend/resume signal.
 *
 * Live SSH sessions keep running — only the vault passphrase is cleared from
 * memory. The user must re-enter their passphrase to access new secrets.
 */
export function useAutoLock() {
  const vault = useVaultStore();
  const settings = useSettingsStore();

  const idleSeconds = ref(0);
  let idleTimer: ReturnType<typeof setInterval> | null = null;
  let lastActivity = Date.now();
  let lastIdleCheck = Date.now();
  let hiddenSince: number | null = null;

  function onActivity() {
    lastActivity = Date.now();
    idleSeconds.value = 0;
  }

  function lockVault(reason: string) {
    if (!vault.unlocked) return;

    console.log(`[auto-lock] locking vault: ${reason}`);
    void vault.lock().catch((error) => {
      console.error("[auto-lock] failed to lock vault", error);
    });
  }

  function checkIdle() {
    const now = Date.now();
    const checkGap = (now - lastIdleCheck) / 1000;
    lastIdleCheck = now;

    // A long timer gap is a defensive signal that the WebView was not running
    // normally. It may be caused by OS suspension or extended backgrounding.
    if (
      vault.unlocked &&
      settings.lockOnSleep &&
      checkGap > BACKGROUND_GAP_SECONDS
    ) {
      lockVault(`extended timer gap (${Math.round(checkGap)}s)`);
      return;
    }

    if (!vault.unlocked || settings.autoLockMinutes <= 0) return;

    const idle = (now - lastActivity) / 1000;
    idleSeconds.value = Math.floor(idle);

    const threshold = settings.autoLockMinutes * 60;
    if (idle >= threshold) {
      lockVault(`inactivity timeout (${settings.autoLockMinutes} min)`);
    }
  }

  function onVisibilityChange() {
    if (document.hidden) {
      hiddenSince = Date.now();
      return;
    }

    const now = Date.now();
    if (hiddenSince !== null) {
      const hiddenDuration = (now - hiddenSince) / 1000;
      hiddenSince = null;

      if (
        hiddenDuration > BACKGROUND_GAP_SECONDS &&
        settings.lockOnSleep
      ) {
        lockVault(`app backgrounded for ${Math.round(hiddenDuration)}s`);
      } else {
        // Evaluate the elapsed idle period before treating the user returning to
        // the app as fresh activity. Otherwise background time could be erased.
        checkIdle();
      }
    } else {
      checkIdle();
    }

    onActivity();
  }

  onMounted(() => {
    lastIdleCheck = Date.now();

    window.addEventListener("mousemove", onActivity, { passive: true });
    window.addEventListener("mousedown", onActivity, { passive: true });
    window.addEventListener("keydown", onActivity, { passive: true });
    window.addEventListener("touchstart", onActivity, { passive: true });
    window.addEventListener("scroll", onActivity, { passive: true });
    document.addEventListener("visibilitychange", onVisibilityChange);

    idleTimer = setInterval(checkIdle, IDLE_CHECK_INTERVAL_MS);
  });

  onUnmounted(() => {
    window.removeEventListener("mousemove", onActivity);
    window.removeEventListener("mousedown", onActivity);
    window.removeEventListener("keydown", onActivity);
    window.removeEventListener("touchstart", onActivity);
    window.removeEventListener("scroll", onActivity);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    if (idleTimer) clearInterval(idleTimer);
  });

  return { idleSeconds };
}
