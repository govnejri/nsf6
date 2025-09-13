import { MapPoint } from "../types/common";
import { HeatmapRectangle, HeatmapRequest, Heatmap, HeatmapResponse } from "../types/heatmap";

export default async function getHeatmap(
	req: HeatmapRequest
): Promise<HeatmapResponse | { error: string }> {
	const response = await fetch(`/api/heatmap?` + new URLSearchParams({
		long: String(req.area.topLeft.long),
	}), {
		method: "GET",
	});
	if (!response.ok) {
		return { error: "Failed to fetch heatmap" };
	}
	const data = await response.json();
	return data as HeatmapResponse;
}

export function makeRequest(topLeft: MapPoint, bottomRight: MapPoint, countX: number, countY: number, timeStart: Date, timeEnd: Date): HeatmapRequest {
	const tileWidth = (bottomRight.long - topLeft.long) / countX;
	const tileHeight = (bottomRight.lat - topLeft.lat) / countY;
	return {
		area: {
			topLeft,
			bottomRight,
		},
		timeStart,
		timeEnd,
		tileWidth,
		tileHeight,
	};
}

export function getMockHeatmap(req: HeatmapRequest): Promise<HeatmapResponse> {
	const { area, tileWidth, tileHeight } = req;
	const latCount = Math.ceil(
		(area.bottomRight.lat - area.topLeft.lat) / tileHeight
	);
	const longCount = Math.ceil(
		(area.bottomRight.long - area.topLeft.long) / tileWidth
	);
	const data: HeatmapRectangle[] = [];
	for (let i = 0; i < latCount; i++) {
		for (let j = 0; j < longCount; j++) {
			const topLeft = {
				lat: area.topLeft.lat + i * tileHeight,
				long: area.topLeft.long + j * tileWidth,
			};
			const bottomRight = {
				lat: Math.min(topLeft.lat + tileHeight, area.bottomRight.lat),
				long: Math.min(topLeft.long + tileWidth, area.bottomRight.long),
			};
			data.push({
				count: Math.floor(Math.random() * 100),
				topLeft,
				bottomRight,
			});
		}
	}
	// 1 sec delay
	return new Promise((resolve) => setTimeout(() => resolve({ heatmap: { data } }), 1000));
}
