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
                    src: 'images/player3.png',
                    type: 'spritesheet',
                    spriteWidth: 25,
                    spriteHeight: 25
                }]);

                this.observe(this.messages, 'map:init', this.setMap, this);

            },

            setMap: function (data) {
                this.map = data.map;
                this.mapView = data.view;
                this.observe(this.mapView, 'tile:click', this.handleTileClick, this);

                var pos = this.map.getStartPos();
                this.x = pos[0];
                this.y = pos[1];

                this.view = this.viewFactory.createView(this, {
                    target: '#map',
                    entity: this,
                    mapView: this.mapView,
                    id: 'player',
                    sheet: this.resources.get('playerSprite'),
                    animations: {
                        'idyl': {
                            frames: [0]
                        },
                        'walk': {
                            frames: [10, 11, 12, 13]
                        }
                    },
                    width: 25,
                    height: 25
                });
            },

            handleTileClick: function (eventData) {
                if (!eventData) {
                    return;
                }
                var ts = Date.now();
                var path = this.map.getPath({
                    col: Math.floor(this.x),
                    row: Math.floor(this.y)
                }, {
                    col: eventData.column,
                    row: eventData.row
                });

                if (!path) {
                    return;
                }

                console.log('Calculated path of length ' + path.length + ' ( ' + (Date.now() - ts) + 'ms)');

                $('.tile.selected').removeClass('selected');
                alchemy.each(path, function (tile) {
                    $('.tile[data-column=' + tile.col + '][data-row=' + tile.row +']').addClass('selected');
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

