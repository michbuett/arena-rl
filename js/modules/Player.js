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
            playerId: 'player',

            init: function () {
                // add mouse listeners to control the player

                // add map listeners
                this.observe(this.messages, 'map:init', this.setMap, this);
            },

            setMap: function (data) {
                this.map = data.map;
                this.mapView = data.view;

                this.observe(this.mapView, 'map:mousedown', this.mouseDownHandler, this);
                this.observe(this.mapView, 'map:mouseup', this.mouseUpHandler, this);
                this.observe(this.mapView, 'map:hover', this.mouseMoveHandler, this);
                //this.observe(this.mapView, 'tile:click', this.handleTileClick, this);

                var pos = this.map.getStartPos();

                this.entities.createEntity(this.playerId, {
                    id: this.playerId,
                    position: {
                        x: pos[0],
                        y: pos[1],
                        direction: pos[2]
                    },
                    view: {
                        entity: this,
                        mapView: this.mapView
                    }
                });

                this.position = this.entities.getComponent('position', this.playerId);
                this.playerInitialized = true;
            },

            mouseDownHandler: function (e) {
                if (e.which === 1) {
                    // lef mouse button down -> start moving
                    this.targetPosition = {
                        x: e.mapX,
                        y: e.mapY
                    };
                }
            },

            mouseUpHandler: function (e) {
                //console.log('MOUSE UP', e);
                if (e.which === 1) {
                    // lef mouse button up stop moving
                    this.targetPosition = null;
                }
            },

            mouseMoveHandler: function (e) {
                var dx = e.mapX - this.position.x;
                var dy = e.mapY - this.position.y;
                var alpha = 180 * Math.atan2(dy, dx) / Math.PI;

                if (alpha < 0) {
                    alpha = -1 * alpha;
                } else {
                    alpha = 360 - alpha;
                }
                this.position.direction = Math.floor(alpha);

                if (this.targetPosition) {
                    this.targetPosition.x = e.mapX;
                    this.targetPosition.y = e.mapY;
                }

                //console.log('MOUSE MOVE', dx, dy, alpha);
            },

            // For taktical player controls
            // handleTileClick: function (eventData) {
            //     if (!eventData) {
            //         return;
            //     }
            //     var path = this.map.getPath({
            //         col: Math.floor(this.position.x),
            //         row: Math.floor(this.position.y)
            //     }, {
            //         col: eventData.column,
            //         row: eventData.row
            //     });

            //     if (!path) {
            //         return;
            //     }

            //     $('.tile.selected').removeClass('selected');
            //     alchemy.each(path, function (tile) {
            //         $('.tile[data-column=' + tile.col + '][data-row=' + tile.row + ']').addClass('selected');
            //     });

            // },


            update: function () {
                if (!this.playerInitialized) {
                    return;
                }
                if (this.targetPosition) {
                    var dx = this.targetPosition.x - this.position.x;
                    var dy = this.targetPosition.y - this.position.y;
                    var delta = Math.max(Math.abs(dx), Math.abs(dy));

                    if (delta < 0.1) {
                        this.playAnimation('idle');
                        return;
                    }

                    var stepSize = 0.03;

                    var newX = this.position.x + stepSize * dx / delta;
                    if (this.map.isBlocked(Math.floor(newX), Math.floor(this.position.y))) {
                        newX = this.position.x;
                    }

                    var newY = this.position.y + stepSize * dy / delta;
                    if (this.map.isBlocked(Math.floor(this.position.x), Math.floor(newY))) {
                        newY = this.position.y;
                    }

                    this.position.x = newX;
                    this.position.y = newY;
                    this.playAnimation('walk');
                } else {
                    this.playAnimation('idle');
                }
            },

            playAnimation: function (newAnim) {
                var view = this.entities.getComponent('view', this.playerId);
                if (view && newAnim !== this.currAnim) {
                    view.play(newAnim);
                    this.currAnim = newAnim;
                }
            }
        }
    });
}());

