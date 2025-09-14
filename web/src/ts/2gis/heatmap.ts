import type * as gis from "@2gis/mapgl/types/index";
import { Heatmap } from "../types/heatmap";


const polygonRemoveCallbacks: (() => void)[] = [];

/**
 * Render heatmap grid onto a given mapgl.Map instance.
 */
export function renderHeatmap(
	mapgl: typeof gis,
	map: gis.Map,
	heatmap: Heatmap
) {
    // Kill old heatmap polygons
    polygonRemoveCallbacks.forEach((cb) => cb());
	polygonRemoveCallbacks.length = 0;

	const maxCount = Math.max(...heatmap.data.map((rect) => rect.count+rect.neighborCount*0.125), 1);
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
			color: getColorForCount(rect.count + rect.neighborCount*0.125, maxCount),
			strokeWidth: 0,
			zIndex: 1,
			interactive: true,
		});
		polygon.on('click', () => {
			alert(`Count: ${rect.count}\nNeighbor count: ${rect.neighborCount}`);
		});
        polygonRemoveCallbacks.push(() => polygon.destroy());
	});
}

function getColorForCount(count: number, maxCount: number): string {
	if (count <= 0.2) return "rgba(0,0,0,0)";
	const ratio = count / maxCount;
    const leftHue = 160;
    const rightHue = 0;
    const hue = leftHue + (rightHue - leftHue) * ratio;
    return `hsl(${Math.floor(hue)}, 100%, 60%, ${ratio * 0.5 + 0.1})`;
}
