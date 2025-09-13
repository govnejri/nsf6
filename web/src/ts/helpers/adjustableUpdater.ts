// filepath: /home/omga/Documents/nsf6/web/src/ts/helpers/adjustableUpdater.ts
/**
 * AdjustableUpdater repeatedly executes a callback every N seconds.
 * The interval (N) can be changed on the fly (e.g., from a slider) without creating overlaps or lag.
 */
export class AdjustableUpdater {
	private callback: () => void | Promise<void>;
	private intervalMs: number;
	private timer: number | null = null;
	private running = false;
	private executing = false; // prevent overlapping async executions
	private disposed = false;

	/**
	 * @param callback Function to run each cycle
	 * @param intervalSeconds Initial interval in seconds
	 */
	constructor(callback: () => void | Promise<void>, intervalSeconds: number) {
		this.callback = callback;
		this.intervalMs = this.normalizeInterval(intervalSeconds) * 1000;
	}

	/** Start the updater (no-op if already running). */
	start(immediate = false) {
		if (this.disposed || this.running) return;
		this.running = true;
		if (immediate) {
			this.run();
		} else {
			this.schedule();
		}
	}

	/** Stop further executions. */
	stop() {
		this.running = false;
		if (this.timer !== null) {
			clearTimeout(this.timer);
			this.timer = null;
		}
	}

	/** Update the interval (seconds). Efficient for rapid slider changes. */
	setIntervalSeconds(seconds: number) {
		const ms = this.normalizeInterval(seconds) * 1000;
		if (ms === this.intervalMs) return; // no change
		this.intervalMs = ms;
		if (this.running) {
			// Restart timing from now using new interval
			if (this.timer !== null) clearTimeout(this.timer);
			this.schedule();
		}
	}

	/** Optionally change the callback at runtime. */
	setCallback(cb: () => void | Promise<void>) {
		this.callback = cb;
	}

	/** Execute immediately (without affecting schedule beyond restarting delay). */
	triggerNow() {
		if (!this.running) return;
		if (this.timer !== null) {
			clearTimeout(this.timer);
			this.timer = null;
		}
		this.run();
	}

	/** Clean up for GC. */
	dispose() {
		this.stop();
		this.disposed = true;
	}

	private normalizeInterval(seconds: number): number {
		if (!isFinite(seconds) || seconds <= 0) return 0.05; // minimum 50ms safeguard
		return seconds;
	}

	private schedule() {
		if (!this.running || this.disposed) return;
		this.timer = window.setTimeout(() => this.run(), this.intervalMs);
	}

	private async run() {
		if (!this.running || this.disposed) return;
		if (this.executing) {
			// If previous async still running, skip this tick and reschedule to avoid piling up.
			this.schedule();
			return;
		}
		this.executing = true;
		try {
			await this.callback();
		} catch (e) {
			// Swallow errors to keep loop alive; could add external error hook.
			console.error("[AdjustableUpdater] callback error", e);
		} finally {
			this.executing = false;
			this.schedule();
		}
	}
}

/* Example usage:
const updater = new AdjustableUpdater(() => fetchData(), 2); // every 2s
updater.start(true); // immediate first run
// On slider change:
updater.setIntervalSeconds(newSeconds);
// To stop:
updater.stop();
*/
