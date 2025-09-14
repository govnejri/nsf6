import { MapRectangle } from "./common";

export type HeatmapRequest = {
	area: MapRectangle;
	timeStart: Date;
	timeEnd: Date;
	tileWidth: number;
	tileHeight: number;
};

export type HeatmapRectangle = {
	count: number;
	neighborCount: number;
} & MapRectangle;

export type HeatmapResponse = {
    heatmap: Heatmap | null;
};

export type Heatmap = {
	data: HeatmapRectangle[];
};
