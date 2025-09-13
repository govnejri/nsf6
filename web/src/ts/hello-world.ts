import $ from "jquery";
import getGL from "./2gis/get";
import { renderHeatmap } from "./2gis/heatmap";
import { getMockHeatmap, makeRequest } from "./api/heatmap";
import { MapPoint } from "./types/common";

const topLeftPoint = { lat: 43.1, long: 76.7 };
const bottomRightPoint = { lat: 43.4, long: 77.0 };
const heatmapProm = getMockHeatmap({
    area: {
        topLeft: topLeftPoint,
        bottomRight: bottomRightPoint,
    },
    timeStart: new Date(Date.now() - 24 * 60 * 60 * 1000),
    timeEnd: new Date(),
    tileWidth: 0.1,
    tileHeight: 0.1,
});

getGL().then((mapgl) => {
    const map = new mapgl.Map("map", {
		center: [76.882, 43.238],
		zoom: 8,
		// Demo-key here, use some backend proxy in prod
		key: "96f35a47-3653-4109-ac5b-1365fe492cc9",
	});

    
    map.on('styleload', () => {
        setInterval(() => {
            const bounds = map.getBounds();
            const topLeft: MapPoint = {
                long: bounds.northEast[0],
                lat: bounds.northEast[1]
            },
            bottomRight: MapPoint = {
                long: bounds.southWest[0],
                lat: bounds.southWest[1]
            };
            const request = makeRequest(topLeft, bottomRight, 10, 10, new Date(Date.now() - 24 * 60 * 60 * 1000), new Date());
            getMockHeatmap(request).then((res) => {
                if (res.heatmap) {
                    renderHeatmap(mapgl, map, res.heatmap);
                }
            });
        }, 500)
	});

    const topLeft = new mapgl.Marker(map, {
        coordinates: [topLeftPoint.long, topLeftPoint.lat],
        rotation: 180,
    });
    const bottomRight = new mapgl.Marker(map, {
        coordinates: [bottomRightPoint.long, bottomRightPoint.lat]
    });
});

heatmapProm.then((res) => {
    $("#output").text(JSON.stringify(res, null, 2));
});