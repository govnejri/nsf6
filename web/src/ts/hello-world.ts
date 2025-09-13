import $ from "jquery";

export function helloWorld() {
    alert("Hello, World!");
}

$('button').on('click', helloWorld);