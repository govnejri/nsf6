import type * as gis from "@2gis/mapgl/types/index";
import { Heatmap } from "../types/heatmap";


const polygonsToRemove: gis.Polygon[] = [];

/**
 * Render heatmap grid onto a given mapgl.Map instance.
 */
export function renderHeatmap(
	mapgl: typeof gis,
	map: gis.Map,
	heatmap: Heatmap
) {
    // Kill old heatmap polygons
    polygonsToRemove.forEach((poly) => poly.destroy());

	const maxCount = Math.max(...heatmap.data.map((rect) => rect.count), 1);
	heatmap.data.forEach((rect, idx) => {
		const polygon = new mapgl.Polygon(map, {
			coordinates: [
				[
					[rect.topLeft.long, rect.topLeft.lat],
					[rect.bottomRight.long, rect.topLeft.lat],
					[rect.bottomRight.long, rect.bottomRight.lat],
					[rect.topLeft.long, rect.bottomRight.lat],
					[rect.topLeft.long, rect.topLeft.lat],
				],
			],
			color: getColorForCount(rect.count, maxCount),
			strokeColor: "rgba(0,0,0,0.1)",
			strokeWidth: 1,
			zIndex: 1,
            userData: { heatmapRemove: true, idx }
		});
        polygonsToRemove.push(polygon);
	});
}

function getColorForCount(count: number, maxCount: number): string {
	if (maxCount <= 0) return "rgba(0,0,0,0)";
	const ratio = count / maxCount;
    const leftHue = 160;
    const rightHue = 0;
    const hue = leftHue + (rightHue - leftHue) * ratio;
    return `hsl(${Math.floor(hue)}, 100%, 50%, 0.5)`;
}
