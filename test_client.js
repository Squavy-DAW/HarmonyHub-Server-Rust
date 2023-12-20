import { io } from "socket.io-client";

const preflight = io("http://127.0.0.1:8000/");

preflight.on("connect", () => {
    console.log("Connected to server");

    preflight.emit("sqw:client_preflight", {}, data => {
        preflight.disconnect();
        data = JSON.parse(data);
        console.log("Server responded with", data);
        const socket = io(`http://127.0.0.1:8000/${data.ns}`);
        socket.on("connect", () => {
            console.log(`Connected to server on namespace '${data.ns}'`);
            socket.disconnect();
        });
    })
});