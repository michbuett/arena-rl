/* global $ */

(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    alchemy.formula.add({
        name: 'arena.modules.Player',
        extend: 'alchemy.core.Oculus',
        requires: [
            'arena.view.Entity'
        ],
        ingredients: [{
            key: 'mod',
            ptype: 'arena.modules.Prima'
        }],
        overrides: {
            col: undefined,
            row: undefined,

            init: function () {
                this.observe(this.messages, 'map:init', this.setMap, this);
            },

            setMap: function (data) {
                this.map = data.map;
                this.mapView = data.view;
                this.observe(this.mapView, 'tile:click', this.handleTileClick, this);

                var pos = this.map.getStartPos();

                this.entities.createEntity('player', {
                    id: 'player',
                    position: {
                        x: pos[0],
                        y: pos[1]
                    },
                    view: {
                        entity: this,
                        mapView: this.mapView
                    }
                });

                this.position = this.entities.getComponent('position', 'player');
            },

            handleTileClick: function (eventData) {
                if (!eventData) {
                    return;
                }
                var path = this.map.getPath({
                    col: Math.floor(this.position.x),
                    row: Math.floor(this.position.y)
                }, {
                    col: eventData.column,
                    row: eventData.row
                });

                if (!path) {
                    return;
                }

                $('.tile.selected').removeClass('selected');
                alchemy.each(path, function (tile) {
                    $('.tile[data-column=' + tile.col + '][data-row=' + tile.row + ']').addClass('selected');
                });

            },
        }
    });
}());

