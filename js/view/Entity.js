(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    alchemy.formula.add({
        name: 'arena.view.Entity',
        extend: 'arena.alchemy.AnimatedEl',
        overrides: {
            width: 32,
            height: 32,

            animations: {
                'idyl': {
                    frames: [0]
                },
                'walk': {
                    frames: [10, 11, 12, 13]
                }
            },

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

            getData: function hocuspocus(_super) {
                return function () {
                    return alchemy.mix(_super.call(this), {
                        x: this.entity.col * this.width,
                        y: this.entity.row * this.height,
                        height: this.height,
                        width: this.width
                    });
                };
            }
        }
    });
}());
