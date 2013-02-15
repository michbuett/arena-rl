(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.Application',
        extend: 'browser.Application',
        requires: [
            'arena.HUD',
            'arena.Map',
            'arena.MapView',
            'arena.Player',
            'arena.EntityView'
        ],

        overrides: {
            prepare: function () {
                this.messages = alchemy('Oculus').brew();
                this.hud = alchemy('arena.HUD').brew({
                    target: '#hud',
                    messages: this.messages
                });

                this.map = alchemy('arena.Map').brew();
                this.mapView = alchemy('arena.MapView').brew({
                    target: '#map',
                    map: this.map,
                    messages: this.messages
                });

                this.player = alchemy('arena.Player').brew({
                    map: this.map
                });
                this.playerView = alchemy('arena.EntityView').brew({
                    id: 'player',
                    target: '#map',
                    entity: this.player,
                    messages: this.messages
                });

                this.messages.trigger('app:start');
            },

            update: function (frame) {
                this.player.update(frame, this);
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

