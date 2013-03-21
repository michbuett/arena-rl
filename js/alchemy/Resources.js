(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * credit to ippa for his javascrip game engine (http://jawsjs.com/)
     *
     * @class arena.alchemy.Resources
     * @extends alchemy.core.Oculus
     */
    alchemy.formula.add({
        name: 'arena.alchemy.Resources',
        alias: 'Resources',
        extend: 'alchemy.core.Oculus',
        requires: ['alchemy.core.Collectum', 'arena.alchemy.SpriteSheet'],

        overrides: {

            // Hash of loaded raw data, URLs are keys
            resources: undefined,

            root: '',

            size: 0,

            fileTypes: {
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
            },

            init: function () {
                this.resources = alchemy('Collectum').brew({
                    idProp: 'src'
                });
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
                    return this.resources.get(src).data;
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
                var res = this.resources.get(src);
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
                var res = this.resources.get(src);
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
             * resources.define("player.png")
             * resources.define(["media/bullet1.png", "media/bullet2.png"])
             * resources.loadAll({onfinish: start_game})
             *
             */
            define: function (cfg) {
                if (alchemy.isArray(cfg)) {
                    alchemy.each(cfg, this.define, this);
                    return;
                }

                if (alchemy.isString(cfg)) {
                    cfg = {src: cfg};
                }

                this.resources.add(alchemy.mix({
                    status: 'waiting',
                    type: this.getType(cfg.src)
                }, cfg));
            },

            /** Load all pre-specified resources */
            loadAll: function (options) {
                this.successCount = 0;
                this.failureCount = 0;

                this.resources.each(function (cfg) {
                    this.load(alchemy.mix(cfg, options));
                }, this);
            },

            /** Load one resource-object, i.e: {src: "foo.png"} */
            load: function (cfg) {
                var data;
                var type = cfg.type;
                var srcUrl = this.root + cfg.src + "?" + alchemy.random(10000000);
                var successCb = this.loadSuccess.bind(this, cfg);
                var failureCB = this.loadFailure.bind(this, cfg);
                /*
                var successCb = (function () {
                    this.loadSuccess(cfg);
                }).bind(this);
                var failureCB = (function () {
                    this.loadFailure(cfg);
                }).bind(this);
                */

                if (!this.resources.contains(cfg.src)) {
                    this.define(cfg);
                }

                switch (type) {
                case 'spritesheet':
                case 'image':
                    data = new Image();
                    data.onload = successCb;
                    data.onerror = failureCB;
                    data.src = srcUrl;
                    break;

                case 'audio':
                    throw 'unsupported file type: AUDIO';

                default:
                    data = new XMLHttpRequest();
                    data.onreadystatechange = successCb;
                    data.open('GET', srcUrl, true);
                    data.send(null);
                    break;
                }

                alchemy.mix(this.resources.get(cfg.src), {
                    status: 'loading',
                    data: data
                });
            },

            /**
             * Callback for all resource-loading.
             * @private
             */
            loadSuccess: function (cfg) {
                var src = cfg.src,
                    resource = this.resources.get(src),
                    type = resource.type.toLowerCase();

                // update status
                resource.status = 'success';

                // Process data depending differently on postfix
                switch (type) {
                case 'spritesheet':
                    resource.data = alchemy('SpriteSheet').brew(alchemy.mix({
                        image: resource.data
                    }, cfg));
                    break;

                case 'json':
                    if (this.readyState !== 4) {
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
                var percent = Math.round((this.successCount + this.failureCount) / this.resources.length * 100);

                if (success) {
                    if (cfg.success) {
                        cfg.success.call(cfg.scope, resource, percent, cfg);
                    }
                } else {
                    if (cfg.failure) {
                        cfg.failure.call(cfg.scope, resource, percent, cfg);
                    }
                }
                alchemy.each(resource.callbacks, function (cbCfg) {
                    cbCfg.callback.call(cbCfg.scope, (success ? resource : null));
                });

                if (percent >= 100) {
                    // When loadAll() is 100%, then call the final callback
                    if (cfg.finished) {
                        cfg.finished.call(cfg.scope, cfg);
                    }
                }
            }
        }
    });
}());

