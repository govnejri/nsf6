/**
 *  @file paint.ts
 *  @description Script for painting routes on the map
 */

import getGL from "./2gis/get";
import $ from "jquery";
import astanaMap from "./helpers/astanaMap";

let isPainting = false;
let canPaint = true;

getGL().then((mapgl) => {
    const map = astanaMap(mapgl);

    map.on("styleload", () => {
        map.on('mousedown', (e) => {
            if ((e.originalEvent as MouseEvent).button !== 0) return; // only left button
            isPainting = true;
        });
        $(document).on('mouseup', () => {
            isPainting = false;
        });
        const coordinates: [number, number][] = [];
        let destroyFunc: (() => void) = () => {};
        map.on('mousemove', (e) => {
            if (isPainting && canPaint) {
                destroyFunc();

                const coord = e.lngLat;
                coordinates.push(coord as [number, number]);
                const line = new mapgl.Polyline(map, {
                    coordinates: coordinates,
                });
                destroyFunc = () => line.destroy();
                canPaint = false;
            }
        });
        setInterval(() => {
            canPaint = true;
        }, 20);
    });
});