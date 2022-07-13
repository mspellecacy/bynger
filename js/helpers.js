// import {bulmaCalendar} from "../external/bulma-cal/bulma-calendar";

export function bc_attach_shim(selector, options) {
    return bulmaCalendar.attach(selector, options);
}

export function bc_value_shim(cal) {
    return cal.value();
}
