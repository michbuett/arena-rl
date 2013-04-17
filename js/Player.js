(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    alchemy.formula.add({
        name: 'arena.Player',
        extend: 'alchemy.core.Oculus',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.ApplicationModule'
        }],
        overrides: {
            col: undefined,
            row: undefined,

            init: function () {
                this.resources.define([{
                    id: 'playerSprite',
                    src: 'images/player2.png',
                    type: 'spritesheet',
                    spriteWidth: 25,
                    spriteHeight: 25
                }]);

                this.observe(this.messages, 'map:init', this.setMap, this);

            },

            setMap: function (data) {
                this.map = data.map;
                this.mapView = data.mapView;

                var pos = this.map.getStartPos();
                this.col = pos[0];
                this.row = pos[1];

                this.view = this.viewFactory.createView(this, {
                    target: '#map',
                    entity: this,
                    id: 'player',
                    sheet: this.resources.get('playerSprite')
                });
            },

            update: function (params) {
                if (this.view) {
                    this.view.update(params);
                }
            }
        }
    });
}());

