import { MapPoint } from "../types/common";

export type TimedPoint = {
	randomized_id: number;
	timestamp: Date;
	spd: number;
	azm: number;
} & MapPoint;

export type PostPoints = { points: TimedPoint[] };

export function makeRequest(
	points: MapPoint[],
	from: Date,
	to: Date
): PostPoints {
	function lerpDate(t: number) {
		return new Date(from.getTime() + t * (to.getTime() - from.getTime()));
	}

	const n = points.length;
	const id = Math.floor(Math.random() * 1e9);
	return {
		points: points.map((point, index) => {
			const curDate = lerpDate(index / (n - 1));
			const velocity = index === 0 ? 0 : Math.random() * 10 + 5; // random velocity between 5 and 15 m/s
			const azimuth =
				index === 0
					? 0
					: points[index - 1].lng === point.lng &&
					  points[index - 1].lat === point.lat
					? 0
					: (Math.atan2(
							point.lat - points[index - 1].lat,
							point.lng - points[index - 1].lng
					  ) *
							180) /
					  Math.PI;
			return {
				randomized_id: id,
				...point,
				timestamp: curDate,
				spd:velocity,
				azm:azimuth,
			};
		}),
	};
}

export async function sendPoints(
	data: PostPoints
): Promise<{ success: boolean; error?: string }> {
	const response = await fetch("/api/points/", {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(data),
	});
	if (!response.ok) {
		return { success: false, error: "Failed to send points" };
	}
	return { success: true };
}
