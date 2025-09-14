import { Map } from "@2gis/mapgl/types";
import type * as gis from "@2gis/mapgl/types/index";

export default function astanaMap(mapgl: typeof gis, disableDragging: boolean): Map {
    return new mapgl.Map("map", {
        center: [71.4272, 51.1655],
        zoom: 14,
        // Demo-key here, use some backend proxy in prod
        key: "96f35a47-3653-4109-ac5b-1365fe492cc9",
        disableDragging,
        zoomControl: false,
    });
}