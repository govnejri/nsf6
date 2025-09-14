import $ from "jquery";

export type TimeSliderOptions = {
	container: string; // CSS selector for the container element
	type: "hours" | "days-of-week";
	onChange: (range: { left: number; right: number }) => void; // Callback when the slider range changes
	initialLeft?: number; // optional initial left index
	initialRight?: number; // optional initial right index
	showModeSelector?: boolean; // optional mode selector UI
	modeSelectorLabels?: { hours: string; "days-of-week": string }; // custom labels for mode selector
};

type TimeSliderState = {
	isDragging: boolean;
	activeHandle: "left" | "right" | null;
	leftValue: number; // logical value (index) for left
	rightValue: number; // logical value (index) for right
	steps: number; // number of discrete steps
};

export class TimeSlider {
	private options: TimeSliderOptions;
	private state: TimeSliderState;
	private container: JQuery<HTMLElement>;
	private slider: JQuery<HTMLElement>;
	private leftHandle: JQuery<HTMLElement>;
	private rightHandle: JQuery<HTMLElement>;
	private rangeBar: JQuery<HTMLElement>;
	private label: JQuery<HTMLElement>;
	private ticks: JQuery<HTMLElement>;
	private resizeHandler: () => void;
	private modeSwitch?: JQuery<HTMLElement>;

	private static DAY_LABELS_RU = ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"];

	constructor(options: TimeSliderOptions) {
		this.options = options;
		const steps = options.type === "hours" ? 24 : 7;
		const defaultLeft =
			options.initialLeft !== undefined ? options.initialLeft : 0;
		const defaultRight =
			options.initialRight !== undefined
				? options.initialRight
				: steps - 1;

		this.state = {
			isDragging: false,
			activeHandle: null,
			leftValue: this.clamp(defaultLeft, 0, steps - 1),
			rightValue: this.clamp(defaultRight, 0, steps - 1),
			steps,
		};

		this.container = $(this.options.container);
		this.container.addClass("time-slider-container");
		this.container.addClass(
			this.options.type === "hours" ? "mode-hours" : "mode-days"
		);

		if (this.options.showModeSelector !== false) {
			this.buildModeSelector();
		}

		this.slider = $(
			'<div class="time-slider" role="group" aria-label="Диапазон времени"></div>'
		);
		this.container.append(this.slider);

		this.leftHandle = $(
			'<div class="time-slider-handle left" tabindex="0" role="slider" aria-label="Ползунок начала" aria-valuemin="0"></div>'
		);
		this.rightHandle = $(
			'<div class="time-slider-handle right" tabindex="0" role="slider" aria-label="Ползунок конца" aria-valuemin="0"></div>'
		);
		this.rangeBar = $(
			'<div class="time-slider-range" aria-hidden="true"></div>'
		);
		this.label = $('<div class="time-slider-label"></div>');
		this.ticks = $(
			'<div class="time-slider-ticks" aria-hidden="true"></div>'
		);

		this.slider.append(
			this.rangeBar,
			this.leftHandle,
			this.rightHandle,
			this.label,
			this.ticks
		);

		this.renderTicks();
		this.updateUI();
		this.bindEvents();

		this.resizeHandler = () => this.updateUI();
		window.addEventListener("resize", this.resizeHandler);
	}

	// PUBLIC API
	getRange() {
		return { left: this.state.leftValue, right: this.state.rightValue };
	}

	setRange(left: number, right: number) {
		const max = this.state.steps - 1;
		this.state.leftValue = this.clamp(Math.min(left, right), 0, max);
		this.state.rightValue = this.clamp(Math.max(left, right), 0, max);
		this.updateUI(true);
	}

	setType(newType: "hours" | "days-of-week") {
		if (newType === this.options.type) return;
		const oldSteps = this.state.steps;
		const oldLeft = this.state.leftValue;
		const oldRight = this.state.rightValue;
		const leftRatio = oldLeft / (oldSteps - 1);
		const rightRatio = oldRight / (oldSteps - 1);
		this.options.type = newType;
		this.state.steps = newType === "hours" ? 24 : 7;
		this.state.leftValue = Math.round(leftRatio * (this.state.steps - 1));
		this.state.rightValue = Math.round(rightRatio * (this.state.steps - 1));
		// Safety clamp
		this.state.leftValue = this.clamp(
			this.state.leftValue,
			0,
			this.state.steps - 1
		);
		this.state.rightValue = this.clamp(
			this.state.rightValue,
			this.state.leftValue,
			this.state.steps - 1
		);
		// Update container mode classes
		this.container
			.removeClass("mode-hours mode-days")
			.addClass(newType === "hours" ? "mode-hours" : "mode-days");
		this.renderTicks();
		this.updateUI(true);
	}

	destroy() {
		window.removeEventListener("resize", this.resizeHandler);
		this.slider.remove();
	}

	// INTERNAL
	private bindEvents() {
		const startDrag = (
			e: JQuery.TriggeredEvent,
			handle: "left" | "right"
		) => {
			e.preventDefault();
			this.state.isDragging = true;
			this.state.activeHandle = handle;
			$(document).on("pointermove.timeSlider", (ev) =>
				this.onPointerMove(ev)
			);
			$(document).on("pointerup.timeSlider pointerleave.timeSlider", () =>
				this.endDrag()
			);
		};

		this.leftHandle.on("pointerdown", (e) => startDrag(e, "left"));
		this.rightHandle.on("pointerdown", (e) => startDrag(e, "right"));

		// Click on track to move closest handle
		this.slider.on("pointerdown", (e) => {
			if (
				$(e.target).is(this.leftHandle) ||
				$(e.target).is(this.rightHandle)
			)
				return;
			const rect = this.slider[0].getBoundingClientRect();
			const percent = ((e.clientX! - rect.left) / rect.width) * 100;
			const value = this.percentToValue(percent);
			const distLeft = Math.abs(value - this.state.leftValue);
			const distRight = Math.abs(value - this.state.rightValue);
			if (distLeft < distRight) {
				this.state.leftValue = this.clamp(
					value,
					0,
					this.state.rightValue
				);
			} else {
				this.state.rightValue = this.clamp(
					value,
					this.state.leftValue,
					this.state.steps - 1
				);
			}
			this.updateUI(true);
		});

		// Keyboard accessibility
		const onKey = (e: JQuery.KeyDownEvent, handle: "left" | "right") => {
			let delta = 0;
			if (e.key === "ArrowLeft" || e.key === "ArrowDown") delta = -1;
			if (e.key === "ArrowRight" || e.key === "ArrowUp") delta = 1;
			if (delta !== 0) {
				e.preventDefault();
				if (handle === "left") {
					this.state.leftValue = this.clamp(
						this.state.leftValue + delta,
						0,
						this.state.rightValue
					);
				} else {
					this.state.rightValue = this.clamp(
						this.state.rightValue + delta,
						this.state.leftValue,
						this.state.steps - 1
					);
				}
				this.updateUI(true);
			}
		};

		this.leftHandle.on("keydown", (e) => onKey(e, "left"));
		this.rightHandle.on("keydown", (e) => onKey(e, "right"));
	}

	private buildModeSelector() {
		const labels = this.options.modeSelectorLabels || {
			hours: "Часы",
			"days-of-week": "Дни",
		};
		this.modeSwitch = $('<div class="time-slider-mode"></div>');
		const btnHours = $(
			'<button type="button" data-mode="hours"></button>'
		).text(labels.hours);
		const btnDays = $(
			'<button type="button" data-mode="days-of-week"></button>'
		).text(labels["days-of-week"]);
		this.modeSwitch.append(btnHours, btnDays);
		this.container.prepend(this.modeSwitch);
		const updateActive = () => {
			this.modeSwitch!.find("button").removeClass("active");
			this.modeSwitch!.find(
				`button[data-mode="${this.options.type}"]`
			).addClass("active");
		};
		updateActive();
		this.modeSwitch.on("click", "button", (e) => {
			const mode = $(e.currentTarget).data("mode");
			if (mode && mode !== this.options.type) {
				this.setType(mode);
				updateActive();
			}
		});
	}

	private onPointerMove(e: JQuery.TriggeredEvent) {
		if (!this.state.isDragging || !this.state.activeHandle) return;
		const ev = e as unknown as PointerEvent;
		const rect = this.slider[0].getBoundingClientRect();
		const percent = ((ev.clientX - rect.left) / rect.width) * 100;
		const value = this.percentToValue(percent);
		if (this.state.activeHandle === "left") {
			this.state.leftValue = this.clamp(value, 0, this.state.rightValue);
		} else {
			this.state.rightValue = this.clamp(
				value,
				this.state.leftValue,
				this.state.steps - 1
			);
		}
		this.updateUI(true);
	}

	private endDrag() {
		this.state.isDragging = false;
		this.state.activeHandle = null;
		$(document).off(
			"pointermove.timeSlider pointerup.timeSlider pointerleave.timeSlider"
		);
	}

	private valueToPercent(value: number): number {
		return (value / (this.state.steps - 1)) * 100;
	}

	private percentToValue(percent: number): number {
		const value = (percent / 100) * (this.state.steps - 1);
		return Math.round(this.clamp(value, 0, this.state.steps - 1));
	}

	private clamp(v: number, min: number, max: number) {
		return Math.min(Math.max(v, min), max);
	}

	private renderTicks() {
		this.ticks.empty();
		for (let i = 0; i < this.state.steps; i++) {
			const label =
				this.options.type === "hours"
					? this.formatHour(i)
					: TimeSlider.DAY_LABELS_RU[i];
			const tick = $("<span></span>")
				.text(label)
				.css("left", this.valueToPercent(i) + "%");
			this.ticks.append(tick);
		}
	}

	private formatHour(h: number) {
		const hh = h.toString().padStart(2, "0");
		return `${hh}:00`;
	}

	private formatRangeLabel() {
		if (this.options.type === "hours") {
			return `${this.formatHour(
				this.state.leftValue
			)} – ${this.formatHour(this.state.rightValue)}`;
		}
		return `${TimeSlider.DAY_LABELS_RU[this.state.leftValue]} – ${
			TimeSlider.DAY_LABELS_RU[this.state.rightValue]
		}`;
	}

	private updateUI(triggerCallback = false) {
		const leftPercent = this.valueToPercent(this.state.leftValue);
		const rightPercent = this.valueToPercent(this.state.rightValue);

		this.leftHandle
			.css("left", `${leftPercent}%`)
			.attr("aria-valuenow", this.state.leftValue)
			.attr("aria-valuemax", this.state.rightValue);
		this.rightHandle
			.css("left", `${rightPercent}%`)
			.attr("aria-valuenow", this.state.rightValue)
			.attr("aria-valuemin", this.state.leftValue);

		this.rangeBar.css({
			left: `${leftPercent}%`,
			width: `${rightPercent - leftPercent}%`,
		});

		const text = this.formatRangeLabel();
		this.label.text(text);
		const rangeCenter = (leftPercent + rightPercent) / 2;
		this.label.css("left", `${rangeCenter}%`);

		if (triggerCallback) {
			this.options.onChange({
				left: this.state.leftValue,
				right: this.state.rightValue,
			});
		}
	}
}
