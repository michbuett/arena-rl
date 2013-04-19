(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    alchemy.formula.add({
        name: 'arena.view.Entity',
        extend: 'arena.alchemy.AnimatedEl',
        overrides: {
            width: 32,
            height: 32,


            defaults: { // default values for each animation
                defaults: { // default values for each frame
                    durration: 200
                }
            },

            init: function hocuspocus(_super) {
                return function () {
                    _super.call(this);

                    this.play('walk');
                };
            },

            getScreenX: function (mapX) {
                return Math.floor(this.mapView.getScreenX(mapX) - this.width / 2);
            },

            getScreenY: function (mapY) {
                return Math.floor(this.mapView.getScreenY(mapY) - this.height / 2);
            },

            getData: function hocuspocus(_super) {
                return function () {
                    return alchemy.mix(_super.call(this), {
                        x: this.getScreenX(this.entity.x),
                        y: this.getScreenY(this.entity.y),
                        height: this.height,
                        width: this.width
                    });
                };
            }
        }
    });
}());

