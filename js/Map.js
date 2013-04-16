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

            tileTypes: {
                '.': {
                    type: 'floor'
                },
                '#': {
                    type: 'wall'
                },
                '~': {
                    type: 'water'
                }
            },

            map: [
                '      ###########      ',
                '    ###..#...#..###    ',
                '   ##....#...#....##   ',
                '  ##.....##.##.....##  ',
                ' ##.................## ',
                ' #...................# ',
                '##......#..~..#......##',
                '#..........~..........#',
                '#..........~..........#',
                '####.......~.......####',
                '#..#.......~.......#..#',
                '#.......~~~~~~~.......#',
                '#..#.......~.......#..#',
                '####.......~.......####',
                '#..........~..........#',
                '#..........~..........#',
                '##......#..~..#......##',
                ' #...................# ',
                ' ##.................## ',
                '  ##.....##.##.....##  ',
                '   ##....#...#....##   ',
                '    ###..#...#..###    ',
                '      ###########      '
            ],

            startPos: [11, 1],

            init: function hocuspocus(_super) {
                return function () {
                    this.observe(this.messages, 'app:start', this.initMap, this);

                    _super.call(this);
                };
            },

            initMap: function () {
                this.tiles = [];
                for (var i = 0; i < this.map.length; i++) {
                    var row = this.map[i];
                    for (var j = 0; j < row.length; j++) {
                        var cfg = this.tileTypes[row.charAt(j)];
                        if (cfg) {
                            this.tiles.push(alchemy.mix({
                                row: i,
                                col: j
                            }, cfg));
                        }
                    }
                }

                this.messages.trigger('map:init', {
                    map: this
                });
            },

            getStartPos: function () {
                return this.startPos;
            }
        }
    });
}());

