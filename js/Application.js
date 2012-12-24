(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.Application',
        extend: 'browser.Application',
        requires: ['arena.HUD', 'arena.Map', 'arena.MapView'],
        overrides: {
            prepare: function () {
                this.messages = alchemy('Oculus').create();
                this.hud = alchemy('arena.HUD').create({
                    target: '#hud',
                    messages: this.messages
                });
                this.map = alchemy('arena.Map').create();
                this.mapView = alchemy('arena.MapView').create({
                    target: '#map',
                    map: this.map
                });

                this.messages.trigger('app:start');
            },

            update: function (frame) {
                this.hud.update(frame, this);
                if (frame > 1000) {
                    this.end();
                }
            },

            draw: function () {
                //this.viewport.draw();
            }
        }
    });
}());

