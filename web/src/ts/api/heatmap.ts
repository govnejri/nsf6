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
	const params = new URLSearchParams();
	params.set("lat1", req.area.topLeft.lat.toString());
	params.set("lng1", req.area.topLeft.lng.toString());
	params.set("lat2", req.area.bottomRight.lat.toString());
	params.set("lng2", req.area.bottomRight.lng.toString());
	params.set("tileWidth", req.tileWidth.toString());
	params.set("tileHeight", req.tileHeight.toString());
	if (req.timeStart) params.set("timeStart", req.timeStart);
	if (req.timeEnd) params.set("timeEnd", req.timeEnd);
	if (req.dateStart) params.set("dateStart", req.dateStart);
	if (req.dateEnd) params.set("dateEnd", req.dateEnd);
	if (req.daysOfWeek && req.daysOfWeek.length > 0)
		params.set("daysOfWeek", req.daysOfWeek.join(","));
	const response = await fetch(`/api/heatmap/?` + params.toString(), {
		method: "GET",
	});
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
	timeStartHour?: number,
	timeEndHour?: number,
	dateStart?: Date,
	dateEnd?: Date,
	daysOfWeek?: number[]
): HeatmapRequest {
	// Make sure topLeft and bottomRight are correctly oriented
	if (topLeft.lat < bottomRight.lat) {
		[topLeft.lat, bottomRight.lat] = [bottomRight.lat, topLeft.lat];
	}
	if (topLeft.lng > bottomRight.lng) {
		[topLeft.lng, bottomRight.lng] = [bottomRight.lng, topLeft.lng];
	}

	const tileWidth = (bottomRight.lng - topLeft.lng) / countX;
	const tileHeight = (topLeft.lat - bottomRight.lat) / countY;

	const request: HeatmapRequest = {
		area: {
			topLeft,
			bottomRight,
		},
		tileWidth,
		tileHeight,
	};

	if (typeof timeStartHour === "number") {
		request.timeStart = `${timeStartHour.toString().padStart(2, "0")}:00`;
	}
	if (typeof timeEndHour === "number") {
		request.timeEnd = `${timeEndHour.toString().padStart(2, "0")}:00`;
	}
	if (dateStart instanceof Date) {
		request.dateStart = dateStart.toISOString().slice(0, 10);
	}
	if (dateEnd instanceof Date) {
		request.dateEnd = dateEnd.toISOString().slice(0, 10);
	}
	if (daysOfWeek && daysOfWeek.length > 0) {
		request.daysOfWeek = daysOfWeek;
	}

	return request;
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
	const lngStep = tileWidth; // usually positive, but handle generically

	const alignedLatStart = alignStart(area.topLeft.lat, latStep);
	const alignedLngStart = alignStart(area.topLeft.lng, lngStep);

	const latEnd = area.bottomRight.lat;
	const lngEnd = area.bottomRight.lng;

	const latContinue = (lat: number) =>
		latStep > 0 ? lat < latEnd : lat > latEnd;
	const lngContinue = (lng: number) =>
		lngStep > 0 ? lng < lngEnd : lng > lngEnd;

	const data: HeatmapRectangle[] = [];
	for (let lat = alignedLatStart; latContinue(lat); lat += latStep) {
		for (let lng = alignedLngStart; lngContinue(lng); lng += lngStep) {
			// Compute rectangle bounds independent of step direction to maintain topLeft (max lat, min lng) & bottomRight (min lat, max lng)
			const nextLat = lat + latStep;
			const nextLng = lng + lngStep;
			const topLat = Math.max(lat, nextLat);
			const bottomLat = Math.min(lat, nextLat);
			const leftLng = Math.min(lng, nextLng);
			const rightLng = Math.max(lng, nextLng);

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
				rightLng < Math.min(area.topLeft.lng, area.bottomRight.lng) ||
				leftLng > Math.max(area.topLeft.lng, area.bottomRight.lng)
			)
				continue;

			data.push({
				count: Math.floor(Math.random() * 100),
				topLeft: { lat: topLat, lng: leftLng },
				bottomRight: { lat: bottomLat, lng: rightLng },
				neighborCount: 0,
			});
		}
	}

	return new Promise((resolve) =>
		setTimeout(() => resolve({ heatmap: { data } }), 200)
	);
}
