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

            prepare: function () {
                this.messages = alchemy('Oculus').brew();
                this.viewFactory = alchemy('arena.ViewFactory').brew();

                this.hud = alchemy('arena.HUD').brew({
                    target: '#hud',
                    app: this,
                    messages: this.messages,
                    factory: this.viewFactory
                });


                this.resources = alchemy('Resources').brew();
                this.resources.define([{
                    src: 'images/player2.png',
                    type: 'spritesheet',
                    spriteWidth: 25,
                    spriteHeight: 25
                }]);

                this.resources.loadAll({
                    success: function (ressource, progress) {
                        console.log('Loading resources... ' + progress + '%');
                    },
                    failure: function (resource, progress, cfg) {
                        console.log('Cannot load resource:' + cfg.src);
                    },
                    finished: function () {
                        console.log('Resource loading completed.');
                        this.initMap();
                    },
                    scope: this
                });
            },

            initMap: function () {
                this.map = alchemy('arena.Map').brew();
                this.player = alchemy('arena.Player').brew({
                    map: this.map
                });

                this.hud.showMap(this.map);
                this.messages.trigger('app:start');
            },

            update: function (params) {
                if (this.map) {
                    this.map.update(params);
                }

                if (params.frame > 1000) {
                    this.end();
                }
            },

            draw: function (params) {
                this.hud.update(params);
            }
        }
    });
}());

