
(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    /**
     * Description
     *
     * @class arena.MapView
     * @extends arena.View
     */
    alchemy.formula.add({
        name: 'arena.MapView',
        extend: 'arena.View',
        overrides: {
            tileWidth: 32,
            tileHeight: 32,
            tileTemplate: [
                '<div',
                ' class="tile <$=data.type $>"',
                ' style="left:<$=data.x$>px; top:<$=data.y$>px;"',
                ' data-column="<$=data.col$>"',
                ' data-row="<$=data.row$>"',
                '></div>'
            ].join(''),

            render: function (ctxt) {
                alchemy.each(this.map.tiles, function (tile, i, ctxt) {
                    ctxt.push(alchemy.render(this.tileTemplate, alchemy.mix({
                        x: this.tileWidth * tile.col,
                        y: this.tileHeight * tile.row
                    }, tile)));
                }, this, [ctxt]);
                return ctxt;
            }
        }
    });
}());

