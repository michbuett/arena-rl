(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.Application',
        extend: 'alchemy.browser.Application',
        requires: [
            'arena.modules.HUD',
            'arena.modules.Map',
            'arena.modules.Player',
            'arena.Entities',
            'arena.view.Factory',
            'arena.alchemy.Resources'
        ],

        overrides: {
            /** @lends arena.Application */

            modules: [{
                potion: 'arena.modules.HUD',
                target: '#hud'
            }, {
                potion: 'arena.modules.Map'
            }, {
                potion: 'arena.modules.Player'
            }],

            /**
             * @name init
             * @methodOf arena.Application
             */
            init: function hocuspocus(_super) {
                return function () {
                    this.messages = alchemy('Oculus').brew();
                    this.resources = alchemy('Resources').brew();
                    this.viewFactory = alchemy('arena.view.Factory').brew();
                    this.entities = alchemy('Entities').brew({
                        messages: this.messages,
                        resources: this.resources
                    });

                    alchemy.each(this.modules, function (item, index) {
                        var potion = item.potion;
                        delete item.potion;

                        this.modules[index] = alchemy(potion).brew(alchemy.mix({
                            app: this,
                            messages: this.messages,
                            resources: this.resources,
                            entities: this.entities,
                            viewFactory: this.viewFactory
                        }, item));
                    }, this);

                    _super.call(this);
                };
            },

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

            update: (function () {
                function updateModule(mod, key, params) {
                    mod.update(params);
                }

                return function (params) {
                    // update application modules
                    alchemy.each(this.modules, updateModule, this, [params]);
                    // update all entities

                    this.entities.update(params);

                    if (params.frame > 1000) { // TODO: remove if app runs
                        this.end();
                    }
                };
            }()),

            draw: (function () {
                function drawModule(mod, key, params) {
                    mod.draw(params);
                }

                return function (params) {
                    alchemy.each(this.modules, drawModule, this, [params]);
                };
            }()),

            /**
             * @name dispose
             * @methodOf arena.Application
             */
            dispose: function hocuspocus(_super) {
                return function () {
                    alchemy.each(this.modules, function (mod) {
                        mod.dispose();
                    }, this);

                    this.viewFactory.dispose();
                    this.resources.dispose();
                    this.messages.dispose();

                    _super.call(this);
                };
            },


            //
            //
            // private helper
            //
            //

            handleResourcesSuccess: function (resource, progress) {
                console.log('Loading resources... ' + progress + '%');
                /**
                 * Fired after a sinle resources has been loaded
                 * @event
                 * @param {Object} data The event data
                 * @param {Object} data.resource The loaded resource
                 * @param {Object} data.progress The overall resource loading progress
                 */
                this.messages.trigger('resource:loaded', {
                    resource: resource,
                    progress: progress
                });
            },

            handleResourcesFailure: function (resource, progress, cfg) {
                console.log('Cannot load resource:' + cfg.src);
            },

            handleResourcesFinished: function () {
                console.log('Resource loading completed.');

                /**
                 * Fired after all resources are loaded
                 * @event
                 */
                this.messages.trigger('app:resourcesloaded');

                // initialize application modules
                alchemy.each(this.modules, function (mod) {
                    mod.prepare();
                }, this);

                /**
                 * Fired after application is ready
                 * @event
                 */
                this.messages.trigger('app:start');
            }
        }
    });
}());

