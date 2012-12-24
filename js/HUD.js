/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     */
    alchemy.formula.add({
        name: 'arena.HUD',
        extend: 'arena.View',
        requires: [],
        overrides: {
            template: [
                '<div id="fps"></div>'
            ].join(''),

            init: function () {
                this.on('rendered', function () {
                    this.$fpsEl = $('#fps');
                }, this);
                _super.call(this);
            },

            update: function (frame, app) {
                if (frame % 100 === 0) {
                    if (this.$fpsEl) {
                        this.$fpsEl.text('FPS: ' + Math.round(app.fps()));
                    }
                }
            },

            draw: function () {
            }
        }
    });
}());

