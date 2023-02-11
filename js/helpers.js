// import {bulmaCalendar} from "../external/bulma-cal/bulma-calendar";

export function bc_attach_shim(selector, options) {
    return bulmaCalendar.attach(selector, options);
}

export function bc_value_shim(cal) {
    return cal.value();
}

// Adapted from: https://stackoverflow.com/questions/3665115/how-to-create-a-file-in-memory-for-user-to-download-but-not-through-server
export function export_file(filename, data, type) {
    const blob = new Blob([data], {type: type});
    if(window.navigator.msSaveOrOpenBlob) {
        window.navigator.msSaveBlob(blob, filename);
    } else {
        const elem = window.document.createElement('a');
        elem.href = window.URL.createObjectURL(blob);
        elem.download = filename;
        document.body.appendChild(elem);
        elem.click();
        document.body.removeChild(elem);
    }
}