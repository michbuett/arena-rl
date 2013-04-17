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
                this.resources = alchemy('Collectum').brew();
            },

            /**
             * Get one resource which has been loaded
             *
             * @param {String} id The resource identifier
             *
             * @return {Object}
             *      the resource object
             */
            get: function (id) {
                if (this.isLoaded(id)) {
                    return this.resources.get(id).data;
                } else {
                    return null;
                }
            },

            /**
             * Return <code>true</code> if src is in the process of loading
             * (but not finished yet)
             *
             * @param {String} id The resource identifier
             *
             * @return {Boolean}
             */
            isLoading: function (id) {
                var res = this.resources.get(id);
                return res && res.status === 'loading';
            },

            /**
             * Return <code>true</code> if src has been loaded completely
             *
             * @param {String} id The resource identifier
             *
             * @return {Boolean}
             */
            isLoaded: function (id) {
                var res = this.resources.get(id);
                return res && res.status === 'success';
            },

            /**
             * Adds one or more resource definition which can be later loaded
             * using {@link arena.alchemy.Resources#loadAll}
             *
             * @param {Object/Array} cfg The resource definition object or an array of those objects
             * @param {String} cfg.id The resource identifier
             * @param {String} cfg.src The source URL
             * @param {String} cfg.type Optional; The resource type (will be determined by the src if omitted)
             * @param {Function} cfg.success Optional; The callback when the resource was loaded successfully
             * @param {Function} cfg.error Optional; The callback when loading the resource failed
             * @param {Object} cfg.scope Optional; The execution context for the callbacks
             *
             * @example
             * resources.define({id: 'my-sprite', src: 'images/sprite.png', success: onLoadCallback});
             * resources.define([{id: 'sprite1', src: 'images/sprite1.png'}, {id: 'sprite2, src: 'images/sprite2.png'}]);
             * resources.loadAll({finished: start_game});
             */
            define: function (cfg) {
                if (alchemy.isArray(cfg)) {
                    alchemy.each(cfg, this.define, this);
                    return;
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
                if (!alchemy.isObject(cfg)) {
                    return;
                }

                var data;
                var type = cfg.type;
                var srcUrl = this.root + cfg.src + "?" + alchemy.random(10000000);
                var successCb = this.loadSuccess.bind(this, cfg);
                var failureCB = this.loadFailure.bind(this, cfg);

                if (!this.resources.contains(cfg.id)) {
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
                    data.onload = successCb;
                    data.onerror = failureCB;
                    data.open('GET', srcUrl, true);
                    data.send(null);
                    break;
                }

                alchemy.mix(this.resources.get(cfg.id), {
                    status: 'loading',
                    data: data
                });
            },


            //
            //
            // private helper methods
            //
            //

            /**
             * Helper method to determine the resource type based on the source URL
             * @private
             */
            getType: function (src) {
                var postfix = (/\.([a-zA-Z0-9]+)$/.exec(src)[1]).toLowerCase();
                return this.fileTypes[postfix] || 'default';
            },

            /**
             * Callback for all resource-loading.
             * @private
             */
            loadSuccess: function (cfg) {
                var resource = this.resources.get(cfg.id),
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
                    if (!resource.data || resource.data.readyState !== 4) {
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
                this.processCallbacks(this.resources.get(cfg.id), false, cfg);
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

