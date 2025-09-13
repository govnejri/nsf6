import { load } from "@2gis/mapgl";

export default function getGL() {
    return load().then((mapgl) => mapgl);
}