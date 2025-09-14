import { MapRectangle } from "./common";

export type HeatmapRequest = {
	area: MapRectangle; // degrees
	tileWidth: number; // degrees
	tileHeight: number;
	timeStart?: string; // 11:00
	timeEnd?: string; // 15:00
	daysOfWeek?: number[]; // 0=Mon ... 6=Sun, optional
	dateStart?: string; // 2023-10-01
	dateEnd?: string; // 2023-10-31
	heatmapType: 'heatmap' | 'trafficmap' | 'speedmap';
};


export type HeatmapRectangle = {
	count: number;
	neighborCount: number;
} & MapRectangle;

export type HeatmapResponse = Record<string, Heatmap>;

export type Heatmap = {
	data: HeatmapRectangle[];
};
