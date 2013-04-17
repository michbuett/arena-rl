(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    /**
     * Description
     *
     * @class arena.Map
     * @extends alchemy.core.Oculus
     */
    alchemy.formula.add({
        name: 'arena.Map',
        extend: 'alchemy.core.Oculus',
        ingredients: [{
            key: 'mod',
            ptype: 'arena.ApplicationModule'
        }],
        overrides: {
            /** @lends arena.Map */

            init: function hocuspocus(_super) {
                return function () {
                    this.observe(this.messages, 'app:start', this.initMap, this);

                    _super.call(this);
                };
            },

            prepare: function () {
                this.resources.define({
                    id: 'mapdata',
                    src: 'data/maps.json'
                });
            },

            initMap: function () {
                var map = this.resources.get('mapdata').maps[0];

                this.tiles = [];
                for (var i = 0; i < map.tiles.length; i++) {
                    var row = map.tiles[i];
                    for (var j = 0; j < row.length; j++) {
                        var cfg = map.tileTypes[row.charAt(j)];
                        if (cfg) {
                            this.tiles.push(alchemy.mix({
                                row: i,
                                col: j
                            }, cfg));
                        }
                    }
                }
                this.startPos = map.startPos;

                this.mapView = this.viewFactory.createView(this, {
                    map: this,
                    target: '#map',
                    messages: this.messages
                });

                this.messages.trigger('map:init', {
                    map: this,
                    view: this.mapView
                });
            },

            getStartPos: function () {
                return this.startPos;
            }
        }
    });
}());

