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
            'arena.HUD',
            'arena.Map',
            'arena.Player',
            'arena.ViewFactory',
            'arena.alchemy.Resources'
        ],

        overrides: {
            /** @lends arena.Application */

            mods: [{
                potion: 'arena.HUD',
                target: '#hud'
            }, {
                potion: 'arena.Map'
            }, {
                potion: 'arena.Player'
            }],

            /**
             * @name init
             * @methodOf arena.Application
             */
            init: function hocuspocus(_super) {
                return function () {
                    this.messages = alchemy('Oculus').brew();
                    this.resources = alchemy('Resources').brew();
                    this.viewFactory = alchemy('arena.ViewFactory').brew();

                    alchemy.each(this.mods, function (item, index) {
                        var potion = item.potion;
                        delete item.potion;

                        this.mods[index] = alchemy(potion).brew(alchemy.mix({
                            app: this,
                            messages: this.messages,
                            resources: this.resources,
                            viewFactory: this.viewFactory
                        }, item));
                    }, this);

                    _super.call(this);
                };
            },

            prepare: function () {
                alchemy.each(this.mods, function (mod) {
                    mod.prepare();
                }, this);


                this.resources.loadAll({
                    success: function (ressource, progress) {
                        console.log('Loading resources... ' + progress + '%');
                    },
                    failure: function (resource, progress, cfg) {
                        console.log('Cannot load resource:' + cfg.src);
                    },
                    finished: function () {
                        console.log('Resource loading completed.');
                        this.messages.trigger('app:start');
                    },
                    scope: this
                });
            },

            update: (function () {
                function updateMod(mod, key, params) {
                    mod.update(params);
                }

                return function (params) {
                    alchemy.each(this.mods, updateMod, this, [params]);

                    if (params.frame > 1000) { // TODO: remove if app runs
                        this.end();
                    }
                };
            }()),

            draw: (function () {
                function drawMod(mod, key, params) {
                    mod.draw(params);
                }

                return function (params) {
                    alchemy.each(this.mods, drawMod, this, [params]);
                };
            }()),

            /**
             * @name dispose
             * @methodOf arena.Application
             */
            dispose: function hocuspocus(_super) {
                return function () {
                    alchemy.each(this.mods, function (mod) {
                        mod.dispose();
                    }, this);

                    this.viewFactory.dispose();
                    this.resources.dispose();
                    this.messages.dispose();

                    _super.call(this);
                };
            }
        }
    });
}());

