// file_reader.js
export function read_file(path) {
    return fetch(path)
        .then((response) => {
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            return response.arrayBuffer();
        })
        .then((buffer) => {
            return new Uint8Array(buffer);
        });
}


// export function read_file(path) {
//     const buffer = fs.readFileSync(path);
//     const uint8Array = new Uint8Array(buffer);
//     return uint8Array;
// }