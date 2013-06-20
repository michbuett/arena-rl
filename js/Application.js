(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     *
     * @class
     * @name arena.Application
     * @extends alchemy.browser.Application
     * @requires arena.modules.HUD
     * @requires arena.modules.Map
     * @requires arena.modules.Player
     * @requires arena.modules.Renderer
     * @requires arena.Entities
     * @requires arena.alchemy.Resources
     */
    alchemy.formula.add({
        name: 'arena.Application',
        extend: 'alchemy.browser.Application',

        requires: [
            'arena.modules.HUD',
            'arena.modules.Map',
            'arena.modules.Player',
            'arena.modules.Renderer',
            'arena.Entities',
            'arena.alchemy.Resources'
        ],

        overrides: {
            /** @lends arena.Application.prototype */

            modules: [
                'arena.modules.HUD',
                'arena.modules.Map',
                'arena.modules.Player',
                'arena.modules.Renderer'
            ],

            /**
             * @function
             */
            init: function hocuspocus(_super) {
                return function () {
                    this.messages = alchemy('Oculus').brew();
                    this.resources = alchemy('Resources').brew();
                    this.entities = alchemy('Entities').brew({
                        messages: this.messages,
                        resources: this.resources
                    });

                    alchemy.each(this.modules, function (item, index) {
                        var potion;
                        var cfg = {
                            app: this,
                            messages: this.messages,
                            resources: this.resources,
                            entities: this.entities,
                        };

                        if (alchemy.isString(item)) {
                            potion = item;
                        } else {
                            potion = item.potion;
                            delete item.potion;
                            cfg = alchemy.mix(cfg, item);
                        }

                        this.modules[index] = alchemy(potion).brew(cfg);
                    }, this);

                    _super.call(this);
                };
            },

            /**
             * Prepares the application:
             * - load configuration
             * - initialize entities
             * - define/load resources
             */
            prepare: function () {
                this.resources.load({
                    src: 'data/app.json',
                }, {
                    success: function (resource) {
                        var cfg = resource.data;

                        // define initial resources
                        this.resources.define(cfg.resources);

                        // initialize entity manager with the loaded component configuration entity archetypes
                        this.entities.initEntityTypes(cfg.entities);

                        // load all defined resources
                        this.resources.loadAll({
                            success: this.handleResourcesSuccess,
                            failure: this.handleResourcesFailure,
                            finished: this.handleResourcesFinished,
                            scope: this
                        });
                    },
                    scope: this
                });
            },

            /**
             * Generic draw method that calls the "update" method of each registered
             * application modules; Can be overridden;
             *
             * @param {Object} params The loop params object
             * @param {Number} params.frame The index of the current frame
             * @param {Number} params.now The current timestamp
             */
            update: (function () {
                function updateModule(mod, key, params) {
                    mod.update(params);
                }

                return function (params) {
                    alchemy.each(this.modules, updateModule, this, [params]);
                };
            }()),

            /**
             * Generic draw method that calls the "draw" method of each registered
             * application modules; Can be overridden;
             *
             * @param {Object} params The loop params object
             * @param {Number} params.frame The index of the current frame
             * @param {Number} params.now The current timestamp
             */
            draw: (function () {
                function drawModule(mod, key, params) {
                    mod.draw(params);
                }

                return function (params) {
                    alchemy.each(this.modules, drawModule, this, [params]);
                };
            }()),

            /**
             * Override super type to dispose modules, resource manager, message bus
             * and entity manager
             * @function
             */
            dispose: function hocuspocus(_super) {
                return function () {
                    alchemy.each(this.modules, function (mod) {
                        mod.dispose();
                    }, this);

                    this.resources.dispose();
                    this.entities.dispose();
                    this.messages.dispose();

                    _super.call(this);
                };
            },


            //
            //
            // private helper
            //
            //

            /**
             * Callback for loading a single resource
             * @private
             */
            handleResourcesSuccess: function (resource, progress) {
                console.log('Loading resources... ' + progress + '%');

                /**
                 * Fired after a single resources has been loaded
                 * @event
                 * @name resource:loaded
                 * @param {Object} data The event data
                 * @param {Object} data.resource The loaded resource
                 * @param {Object} data.progress The overall resource loading progress
                 */
                this.messages.trigger('resource:loaded', {
                    resource: resource,
                    progress: progress
                });
            },

            /**
             * Callback in case a resource cannot be loaded
             * @private
             */
            handleResourcesFailure: function (resource, progress, cfg) {
                console.log('Cannot load resource:' + cfg.src);

                /**
                 * Fired after a single resources has been failed to loaded
                 * @event
                 * @name resource:error
                 * @param {Object} data The event data
                 * @param {Object} data.resource The resource configuration which failed to load
                 */
                this.messages.trigger('resource:loaded', {
                    resource: resource,
                    progress: progress
                });

            },

            /**
             * Callback in case all resources are loaded
             * @private
             */
            handleResourcesFinished: function () {
                console.log('Resource loading completed.');

                /**
                 * Fired after all resources are loaded
                 * @event
                 * @name app:resourcesloaded
                 */
                this.messages.trigger('app:resourcesloaded');

                // initialize application modules
                alchemy.each(this.modules, function (mod) {
                    mod.prepare();
                }, this);

                /**
                 * Fired after application is ready
                 * @event
                 * @name app:start
                 */
                this.messages.trigger('app:start');
            }
        }
    });
}());

