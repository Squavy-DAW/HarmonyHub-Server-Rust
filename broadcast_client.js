import { io } from "socket.io-client";
import { argv } from "node:process";

const enc = new TextEncoder();

if (argv.length < 3) {
    console.log("connecting with preflight");
    const preflight = io("http://127.0.0.1:8000/");

    preflight.on("connect", () => {
        console.log("Connected to server");

        preflight.emit("sqw:client_preflight", {}, data => {
            console.log("Your namespace: ", data.ns);
            preflight.disconnect();

            const socket = io(`http://127.0.0.1:8000/${data.ns}`);
            socket.on("sqw:data", (...args) => {
                const callback = args.pop()
                console.log("sqw:data response: ", JSON.stringify(args));
                callback(enc.encode("Very nice callback!"))
            });
        })
    });
}
else {
    console.log("connecting to namespace", argv[2]);
    const socket = io(`http://127.0.0.1:8000/${argv[2]}`);

    socket.on("connect", () => {
        socket.emit("sqw:broadcast", enc.encode("Hello world!"), (...response) => {
            console.log("sqw:broadcast response: ", JSON.stringify(response));
        });
    })

    socket.on("sqw:data", (...args) => {
        const callback = args.pop()
        console.log("sqw:data response: ", JSON.stringify(args));
        callback(enc.encode("Very nice callback also!"))
    });
}