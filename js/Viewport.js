/*global $*/
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');

    /**
     * Description
     *
     * @class browser.Viewport
     * @extends MateriaPrima
     */
    alchemy.formula.add({
        name: 'arena.Viewport',
        extend: 'arena.View',
        requires: [],
        overrides: {
            template: [
                /*jshint white: false*/
                '<div id="viewport">',
                    '<div id="fps"></div>',
                    '<div id="hud"></div>',
                    '<div id="map"></div>',
                '</div>'
                /*jshint white: true*/
            ].join(''),

            init: function () {
                _super.call(this);

                this.on('rendered', function () {
                    this.$fpsEl = $('#fps');
                    this.$hudEl = $('#hud');
                    this.$mapEl = $('#map');
                }, this);
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

