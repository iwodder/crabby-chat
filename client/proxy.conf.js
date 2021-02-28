const PROXY_CONFIG = {
    "/room/*": {
        "target": "http://[::1]:8000",
        "secure": false,
        "logLevel": "debug",
        "bypass": function (req, res, proxyOptions) {
            console.log(req.url);
        }
    }
}

module.exports = PROXY_CONFIG;