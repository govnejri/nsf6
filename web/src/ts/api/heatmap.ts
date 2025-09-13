import { MapPoint } from "../types/common";
import {
	HeatmapRectangle,
	HeatmapRequest,
	Heatmap,
	HeatmapResponse,
} from "../types/heatmap";

export default async function getHeatmap(
	req: HeatmapRequest
): Promise<HeatmapResponse | { error: string }> {
	const response = await fetch(
		`/api/heatmap?` +
			new URLSearchParams({
				long: String(req.area.topLeft.long),
			}),
		{
			method: "GET",
		}
	);
	if (!response.ok) {
		return { error: "Failed to fetch heatmap" };
	}
	const data = await response.json();
	return data as HeatmapResponse;
}

export function makeRequest(
	topLeft: MapPoint,
	bottomRight: MapPoint,
	countX: number,
	countY: number,
	timeStart: Date,
	timeEnd: Date
): HeatmapRequest {
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
	// Support both positive and negative tile sizes (depending on coordinate orientation)
	if (tileWidth === 0 || tileHeight === 0) {
		return Promise.resolve({ heatmap: { data: [] } });
	}

	const alignStart = (value: number, step: number) => {
		return step > 0
			? Math.floor(value / step) * step
			: Math.ceil(value / step) * step;
	};

	const latStep = tileHeight; // may be negative
	const longStep = tileWidth; // usually positive, but handle generically

	const alignedLatStart = alignStart(area.topLeft.lat, latStep);
	const alignedLongStart = alignStart(area.topLeft.long, longStep);

	const latEnd = area.bottomRight.lat;
	const longEnd = area.bottomRight.long;

	const latContinue = (lat: number) =>
		latStep > 0 ? lat < latEnd : lat > latEnd;
	const longContinue = (lng: number) =>
		longStep > 0 ? lng < longEnd : lng > longEnd;

	const data: HeatmapRectangle[] = [];
	for (let lat = alignedLatStart; latContinue(lat); lat += latStep) {
		for (let lng = alignedLongStart; longContinue(lng); lng += longStep) {
			// Compute rectangle bounds independent of step direction to maintain topLeft (max lat, min long) & bottomRight (min lat, max long)
			const nextLat = lat + latStep;
			const nextLong = lng + longStep;
			const topLat = Math.max(lat, nextLat);
			const bottomLat = Math.min(lat, nextLat);
			const leftLong = Math.min(lng, nextLong);
			const rightLong = Math.max(lng, nextLong);

			// Skip tiles that fall completely outside requested area (in case alignment extended outward)
			if (
				topLat < Math.min(area.topLeft.lat, area.bottomRight.lat) &&
				bottomLat < Math.min(area.topLeft.lat, area.bottomRight.lat)
			)
				continue;
			if (
				bottomLat > Math.max(area.topLeft.lat, area.bottomRight.lat) &&
				topLat > Math.max(area.topLeft.lat, area.bottomRight.lat)
			)
				continue;
			if (
				rightLong <
					Math.min(area.topLeft.long, area.bottomRight.long) ||
				leftLong > Math.max(area.topLeft.long, area.bottomRight.long)
			)
				continue;

			data.push({
				count: Math.floor(Math.random() * 100),
				topLeft: { lat: topLat, long: leftLong },
				bottomRight: { lat: bottomLat, long: rightLong },
			});
		}
	}

	return new Promise((resolve) =>
		setTimeout(() => resolve({ heatmap: { data } }), 200)
	);
}
