Alchemy.ns('Alchemy.util');

/**
 * credit to ippa for his javascrip game engine (http://jawsjs.com/)
 */
Alchemy.util.ResourceMgr = {
    // Hash of loaded raw data, URLs are keys
    resources: undefined,

    root: '',

    size: 0,

    constructor: function (cfg) {
        cfg = cfg || {};

        this.resources = {};
        this.fileTypes = {
            json: 'json',
            wav: 'audio',
            ogg: 'audio',
            png: 'image',
            jpg: 'image',
            jpeg: 'image',
            gif: 'image',
            bmp: 'image',
            tiff: 'image',
            mp3: 'audio'
        };
        _super.call(this, cfg);
    },

    /**
     * Get one resource which has been loaded
     *
     * @param {String} src
     *      the source URL string
     *
     * @return {Object}
     *      the resource object
     */
    get: function (src) {
        if (this.isLoaded(src)) {
            return this.resources[src].data;
        } else {
            return null;
        }
    },

    /**
     * Return <code>true</code> if src is in the process of loading
     * (but not finished yet)
     *
     * @param {String} src
     *      the src URL
     *
     * @return {Boolean}
     */
    isLoading: function (src) {
        var res = this.resources[src];
        return res && res.status === 'loading';
    },

    /**
     * Return <code>true</code> if src has been loaded completely
     *
     * @param {String} src
     *      the src URL
     *
     * @return {Boolean}
     */
    isLoaded: function (src) {
        var res = this.resources[src];
        return res && res.status === 'success';
    },

    getPostfix: function (src) {
        return (/\.([a-zA-Z0-9]+)$/.exec(src)[1]).toLowerCase();
    },

    getType: function (src) {
        return this.fileTypes[this.getPostfix(src)] || 'default';
    },

    /**
     * Add array of paths or single path to resource-list. Later load with loadAll()
     *
     * @example
     * Alchemy.resources.define("player.png")
     * Alchemy.resources.define(["media/bullet1.png", "media/bullet2.png"])
     * Alchemy.resources.loadAll({onfinish: start_game})
     *
     */
    define: function (cfg) {
        if (Array.isArray(cfg)) {
            cfg.forEach(this.define, this);
        } else {
            if (Alchemy.isString(cfg)) {
                cfg = {src: cfg};
            }
            this.resources[cfg.src] = Alchemy.mix({
                status: 'waiting',
                type: this.getType(cfg.src)
            }, cfg);
            this.size++;
        }
    },

    /** Load all pre-specified resources */
    loadAll: function (options) {
        this.successCount = 0
        this.failureCount = 0

        for (var src in this.resources) {
            if (this.resources.hasOwnProperty(src)) {
                this.load(Alchemy.mix(this.resources[src], options));
            }
        }
    },

    /** Load one resource-object, i.e: {src: "foo.png"} */
    load: function (cfg) {
        var data,
            type = cfg.type,
            srcUrl = this.root + cfg.src + "?" + Alchemy.random(10000000),
            successCb = (function () {
                this.loadSuccess(cfg);
            }).bind(this),
            failureCB = (function () {
                this.loadFailure(cfg);
            }).bind(this);

        if (!this.resources[cfg.src]) {
            this.define(cfg);
        }

        switch (type) {
        case 'spritesheet':
        case 'image':
            data = new Image()
            data.onload = successCb;
            data.onerror = failureCB;
            data.src = srcUrl;
            break;

        case 'audio':
            throw 'unsupported file type: AUDIO';
            break;

        default:
            data = new XMLHttpRequest();
            data.onreadystatechange = successCb;
            data.open('GET', srcUrl, true);
            data.send(null);
            break;
        }

        Alchemy.mix(this.resources[cfg.src], {
            status: 'loading',
            data: data
        });
    },

    /** @private
     * Callback for all resource-loading.
     */
    loadSuccess: function (cfg) {
        var src = cfg.src,
            resource = this.resources[src],
            type = resource.type.toLowerCase();

        // update status
        resource.status = 'success';

        // Process data depending differently on postfix
        switch (type) {
        case 'spritesheet':
            resource.data = Alchemy.v.SpriteSheet.create(Alchemy.mix({
                image: resource.data
            }, cfg));
            break;

        case 'json':
            if (this.readyState != 4) {
                return;
            }
            resource.data = JSON.parse(resource.data.responseText);
            break;

        case 'audio':
            //resource.data.removeEventListener("canplay", ?, false);
            break;
        }

        this.successCount++;
        this.processCallbacks(resource, true, cfg);
    },

    /** @private */
    loadFailure: function (cfg) {
        this.failureCount++;
        this.processCallbacks(this.resources[cfg.src], false, cfg);
    },

    /** @private */
    processCallbacks: function (resource, success, cfg) {
        var percent = Math.round((this.successCount + this.failureCount) / this.size * 100),
            i, cbCfg;

        if (success) {
            if (cfg.success) {
                cfg.success.call(cfg.scope, resource, percent, cfg);
            }

        } else {
            if (cfg.failure) {
                cfg.failure.call(cfg.scope, resource, percent, cfg);
            }
        }
        if (resource.callbacks) {
            for (i = 0; i < resource.callbacks.length; i++) {
                cbCfg = resource.callbacks[i];
                cbCfg.callback.call(cbCfg.scope, (success ? resource : null));
            }
        }

        // When loadAll() is 100%, call onfinish() and kill callbacks (reset with next loadAll()-call)
        if (percent >= 100) {
            if (cfg.finished) {
                cfg.finished.call(cfg.scope, cfg);
            }
        }
    }
};

Alchemy.util.ResourceMgr = Alchemy.brew({
    name: 'ResourceMgr',
    ns: 'Alchemy.util',
    extend: Alchemy.util.Observable
}, Alchemy.util.ResourceMgr);
Alchemy.resources = Alchemy.util.ResourceMgr.create();
