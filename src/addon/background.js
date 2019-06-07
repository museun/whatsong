var active_id = undefined;

const timestamp = () => Math.round((new Date()).getTime() / 1000);

const set_inactive = id => {
    active_id = undefined;
    browser.browserAction.setIcon({ path: "icons/icon.svg", tabId: id });
    browser.tabs.onUpdated.removeListener(listener);
}

const set_active = id => {
    active_id = id;
    browser.browserAction.setIcon({ path: "icons/icon-active.svg", tabId: id });
    const filter = { tabId: id }
    browser.tabs.onUpdated.addListener(listener, filter);
}

const log_url = (url, title) => {
    let ts = timestamp();
    console.log(ts, url, title);

    browser.runtime.sendNativeMessage("whatsong", {
        ts: ts,
        url: url,
        title: title,
    }).then(msg => {
        console.log(msg)
    }, err => {
        console.error(err)
    });
};

function listener(id, info, tab) {
    if (info.url) {
        browser.browserAction.setIcon({ path: "icons/icon-active.svg", tabId: id });
        log_url(tab.url, tab.title);
    }
}

browser.browserAction.onClicked.addListener(tab => {
    if (active_id === tab.id) {
        console.log("setting inactive:", active_id);
        set_inactive(active_id);
        return;
    }

    if (active_id) {
        console.log("setting inactive:", active_id);
        set_inactive(active_id);
    }
    console.log("setting active:", tab.id);
    set_active(tab.id)
});

