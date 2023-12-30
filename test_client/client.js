import { io } from "socket.io-client";
import { argv } from "node:process";

const preflight = io("http://127.0.0.1:8000/");

!argv[2] && preflight.on("connect", () => {
    preflight.emit("sqw:client_preflight", {}, data => {
        console.log("Your namespace:", data.ns);
        preflight.disconnect();
    })
});

const enc = new TextEncoder();
const socket = io(`http://127.0.0.1:8000/${argv[2]}`);

socket.on("connect", () => {
    socket.emit("sqw:broadcast", {nice: "very cool!"}, enc.encode("Some nice broadcasting yk =)"), (...response) => {
        console.log("sqw:broadcast response:", JSON.stringify(response));
    });

    socket.emit("sqw:request", "hello!", enc.encode("Some really cool request"), (...response) => {
        console.log("sqw:request response:", JSON.stringify(response));
    });
})

socket.on("sqw:data", (...args) => {
    const callback = args.pop();
    console.log("sqw:data response:", JSON.stringify(args));
    callback(enc.encode(Math.random().toFixed(3)))
});