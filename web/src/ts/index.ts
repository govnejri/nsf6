import $ from "jquery";
import getGL from "./2gis/get";
import { renderHeatmap } from "./2gis/heatmap";
import getHeatmap, { getMockHeatmap, makeRequest } from "./api/heatmap";
import { MapPoint } from "./types/common";
import { AdjustableUpdater } from "./helpers/adjustableUpdater";


function getUpdateInterval(): number {
    return parseInt($('#update-interval').val() as string) || 1000;
}

let updater: AdjustableUpdater | null = null;

getGL().then((mapgl) => {
    const map = new mapgl.Map("map", {
		center: [76.882, 43.238],
		zoom: 8,
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
			const request = makeRequest(
				topLeft,
				bottomRight,
				24,
				14,
				new Date(Date.now() - 24 * 60 * 60 * 1000),
				new Date()
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