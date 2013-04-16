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
        ingredients: [{
            key: 'mod',
            ptype: 'arena.ApplicationModule'
        }],
        overrides: {
            template: [
                '<div id="fps"></div>'
            ].join(''),

            init: function hocuspocus(_super) {
                return function () {
                    this.on('rendered', function () {
                        this.$fpsEl = $('#fps');
                    }, this);

                    this.observe(this.messages, 'map:init', function (data) {
                        this.showMap(data.map);
                    }, this);

                    _super.call(this);
                };
            },

            update: function (params) {
                if (params.frame % 100 === 0) {
                    if (this.$fpsEl) {
                        this.$fpsEl.text('FPS: ' + Math.round(this.app.fps()));
                    }
                }

                if (this.map) {
                    this.map.update(params);
                }
            },

            showMap: function (map) {
                this.map = this.viewFactory.createView(map, {
                    map: map,
                    target: '#map',
                    messages: this.messages
                });
            },


            dispose: function hocuspocus(_super) {
                return function () {
                    this.map.dispose();

                    delete this.map;
                    delete this.app;
                    delete this.$fpsEl;

                    _super.call(this);
                };
            }
        }
    });
}());

