import $ from "jquery";
import getGL from "./2gis/get";
import { renderHeatmap } from "./2gis/heatmap";
import getHeatmap, { getMockHeatmap, makeRequest } from "./api/heatmap";
import { MapPoint } from "./types/common";
import { AdjustableUpdater } from "./helpers/adjustableUpdater";
import { TimeSlider } from "./components/timeSlider";


function getUpdateInterval(): number {
    return parseInt($('#update-interval').val() as string) || 1000;
}

let updater: AdjustableUpdater | null = null;

const timeSlider = new TimeSlider({
	type: 'hours',
	container: '#time-slider',
	onChange: (range) => {}
});

getGL().then((mapgl) => {
    const map = new mapgl.Map("map", {
		center: [71.4272, 51.1655],
		zoom: 14,
		// Demo-key here, use some backend proxy in prod
		key: "96f35a47-3653-4109-ac5b-1365fe492cc9",
	});

    
    map.on('styleload', () => {
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
			const {left: leftValue, right: rightValue} = timeSlider.getRange(); // 0-100 (%)
			// Map to hours of today
			const now = new Date();
			const startHour = Math.floor((leftValue / 100) * 24);
			const endHour = Math.ceil((rightValue / 100) * 24);
			const startTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), startHour, 0, 0);
			const endTime = new Date(now.getFullYear(), now.getMonth(), now.getDate(), endHour, 0, 0);

			const request = makeRequest(
				topLeft,
				bottomRight,
				96, 
				54,
				startTime,
				endTime
			);
			getHeatmap(request).then((res) => {
				if ('error' in res) {
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

$('#update-interval').on('input', (ev) => {
    $('#update-interval-value').text(getUpdateInterval());
    if (updater) {
		updater.setIntervalSeconds(getUpdateInterval() / 1000);
	}
});
$('#update-interval-value').text(getUpdateInterval());