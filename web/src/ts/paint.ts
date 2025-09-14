/**
 *  @file paint.ts
 *  @description Script for painting routes on the map
 */

import getGL from "./2gis/get";
import $ from "jquery";
import astanaMap from "./helpers/astanaMap";
import { makeRequest, sendPoints } from "./api/sendPoints";

let isPainting = false;
let canPaint = true;

getGL().then((mapgl) => {
    const map = astanaMap(mapgl,true);
    const coordinates: [number, number][] = [];
    let destroyFunc: (() => void) = () => {};

    map.on("styleload", () => {
        map.on('mousedown', (e) => {
            if ((e.originalEvent as MouseEvent).button !== 0) return; // only left button
            isPainting = true;
        });
        $(document).on('mouseup', () => {
            isPainting = false;
        });
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

    $('#clear').on('click', () => {
        destroyFunc();
        coordinates.length = 0;
    })
    $('#send').on('click', () => {
        if ($('#start-datetime').val() === '' || $('#end-datetime').val() === '') {
            alert('Please select start and end datetime');
            return;
        }
        if (coordinates.length < 2) {
            alert('Please draw a route with at least 2 points');
            return;
        }
        sendPoints(makeRequest(coordinates.map(c => ({ lng: c[0], lat: c[1] })), new Date($('#start-datetime').val() as string), new Date($('#end-datetime').val() as string))).then(res => {
            if (res.success) {
                alert('Points sent successfully');
                destroyFunc();
                coordinates.length = 0;
            } else {
                alert('Error sending points: ' + res.error);
            }
        });
    });


});