(function () {
    'use strict';

    var alchemy = require('./Alchemy.js');

    /**
     * Description
     *
     * @class
     * @name arena.modules.Locator
     * @extends alchemy.core.Oculus
     */
    alchemy.formula.add({
        name: 'arena.modules.Locator',
        extend: 'alchemy.core.Oculus',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.modules.Prima'
        }],

        overrides: {
            /** @lends arena.modules.Locator.prototype */

            init: function () {
                this.observe(this.messages, 'map:init', function (data) {
                    this.map = data.map;
                    this.mapView = data.view;
                }, this);
            },

            update: function (params) {
                var positions = this.entities.getComponent('position');
                if (positions) {
                    positions.each(this.updatePosition, this, [params]);
                }
            },

            updatePosition: function (position) {
                var x = position.x, y = position.y;
                var col = Math.floor(x), row = Math.floor(y);
                var target = position.target;

                if (target) {
                    var dx = target.x - x;
                    var dy = target.y - y;
                    var delta = Math.max(Math.abs(dx), Math.abs(dy));

                    if (delta < 0.1) {
                        delete position.target;
                        return;
                    }


                    var currentFrame = path[0];
                    if (col !== currentFrame.col || row !== currentFrame.row) {
                        path.shift();
                        position.target = {
                            x: path[0].col + 0.5,
                            y: path[0]
                        }
                    }
                }
            },

            dispose: function hocuspocus(_super) {
                return function () {
                    delete this.map;
                    delete this.mapView;

                    _super.call(this);
                };
            }
        }
    });
}());
