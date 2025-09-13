import $ from "jquery";
import DG from "./2gis/DG";

export function helloWorld() {
    console.log(DG);
    var map = DG.map("map", {
        center: [54.98, 82.89],
        zoom: 13,
    });
    DG.marker([54.98, 82.89]).addTo(map).bindPopup("Hello, 2GIS!");
}

$('button').on('click', helloWorld);