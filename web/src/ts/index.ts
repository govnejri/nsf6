import $ from "jquery";
import getGL from "./2gis/get";
import { renderHeatmap } from "./2gis/heatmap";
import getHeatmap, { getMockHeatmap, makeRequest } from "./api/heatmap";
import { MapPoint } from "./types/common";
import { AdjustableUpdater } from "./helpers/adjustableUpdater";
import { TimeSlider } from "./components/timeSlider";
import astanaMap from "./helpers/astanaMap";

function getUpdateInterval(): number {
	return parseInt($("#update-interval").val() as string) || 1000;
}

let updater: AdjustableUpdater | null = null;

const timeSlider = new TimeSlider({
	type: "hours",
	container: "#time-slider",
	showModeSelector: true,
	onChange: (range) => {},
});

getGL().then((mapgl) => {
	const map = astanaMap(mapgl);

	map.on("styleload", () => {
		updater = new AdjustableUpdater(async () => {
			const bounds = map.getBounds();
			const topLeft: MapPoint = {
					long: bounds.northEast[0],
					lat: bounds.northEast[1],
				},
				bottomRight: MapPoint = {
					long: bounds.southWest[0],
					lat: bounds.southWest[1],
				};
			const { left: leftValue, right: rightValue } =
				timeSlider.getRange(); // indices
			// If current type is hours -> indices map 1:1; if days -> treat as 24h window around selected days (simple example)
			let startHour: number;
			let endHour: number;
			if ((timeSlider as any).options?.type === "hours") {
				startHour = leftValue;
				endHour = rightValue; // inclusive -> exclusive
			} else {
				// days-of-week: approximate to whole days (multiply by 24)
				startHour = leftValue * 24;
				endHour = rightValue * 24;
			}

			const request = makeRequest(
				topLeft,
				bottomRight,
				96,
				54,
				startHour,
				endHour
			);
			getHeatmap(request).then((res) => {
				if ("error" in res) {
					console.error("Heatmap error:", res.error);
					return;
				}
				if (res.heatmap) {
					renderHeatmap(mapgl, map, res.heatmap);
				}
			});
		}, getUpdateInterval() / 1000);
		updater.start(true);
	});
});

$("#update-interval").on("input", (ev) => {
	$("#update-interval-value").text(getUpdateInterval());
	if (updater) {
		updater.setIntervalSeconds(getUpdateInterval() / 1000);
	}
});
$("#update-interval-value").text(getUpdateInterval());
